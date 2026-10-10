//! Scratch remote-and-clone pairs for tests that exercise a real pull.
//!
//! Every helper here builds a bare remote plus one or more clones of it in
//! fresh tempdirs, so a test can drive a pull path against real git history
//! without touching any repository it does not own.

use std::path::Path;

use crate::{clone_repo, git, init_bare_repo};

fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().expect("git-fixture: cannot create a tempdir")
}

fn write(path: &Path, contents: &str) {
    std::fs::write(path, contents)
        .unwrap_or_else(|e| panic!("git-fixture: cannot write {path:?}: {e}"));
}

/// Build a scratch bare remote and one clone that tracks it.
/// Returns `(remote_tempdir, clone_tempdir)`. The remote's `main` holds one
/// commit and the clone is checked out at it, so a pull from the clone has
/// nothing to fetch.
pub fn scratch_remote_and_clone() -> (tempfile::TempDir, tempfile::TempDir) {
    let remote = tempdir();
    init_bare_repo(remote.path());

    // Seed the remote via an intermediate clone, then hand back a second
    // fresh clone. Using two separate clones prevents the seed push from
    // accidentally advancing the caller's local HEAD.
    // Clone into the live tempdir (git accepts an existing empty directory).
    // Deleting it first would free tempfile's name reservation, letting a
    // concurrent test process draw the same path and collide.
    let seed = tempdir();
    clone_repo(remote.path(), seed.path());
    write(&seed.path().join("readme.md"), "hello");
    git(seed.path(), &["add", "."]);
    git(seed.path(), &["commit", "-m", "initial"]);
    git(seed.path(), &["push", "-u", "origin", "main"]);

    let clone = tempdir();
    clone_repo(remote.path(), clone.path());
    (remote, clone)
}

/// Same as `scratch_remote_and_clone` but pushes one extra commit from a
/// sibling clone before returning, so the remote's `main` is one commit ahead
/// of the clone's HEAD. The clone's `origin/main` remote-tracking ref still
/// names the clone's HEAD until a fetch; a pull on the clone fetches and
/// fast-forwards its HEAD.
pub fn scratch_remote_and_clone_behind_by_one() -> (tempfile::TempDir, tempfile::TempDir) {
    let (remote, clone) = scratch_remote_and_clone();
    push_sibling_commit(remote.path(), "upstream advance");
    (remote, clone)
}

/// Push one commit from a fresh sibling clone to advance the remote's `main`.
/// Returns the pushed commit's SHA.
///
/// The commit writes `x` to `<message>.txt`, so `message` must be usable as a
/// file name. A repeat call with the same message changes no file and pushes
/// an empty commit.
pub fn push_sibling_commit(remote: &Path, message: &str) -> String {
    let sibling = tempdir();
    clone_repo(remote, sibling.path());
    write(&sibling.path().join(format!("{message}.txt")), "x");
    git(sibling.path(), &["add", "."]);
    git(sibling.path(), &["commit", "--allow-empty", "-m", message]);
    git(sibling.path(), &["push", "origin", "main"]);
    head(sibling.path())
}

/// The SHA of `HEAD` in `path`.
pub fn head(path: &Path) -> String {
    git(path, &["rev-parse", "HEAD"]).trim().to_string()
}

/// Make a local commit in `path` without pushing.
///
/// The commit writes the literal text "content" to `filename`. A repeat call
/// with the same filename changes no file and makes an empty commit.
pub fn local_commit(path: &Path, filename: &str, message: &str) {
    write(&path.join(filename), "content");
    git(path, &["add", "."]);
    git(path, &["commit", "--allow-empty", "-m", message]);
}

/// Create an orphan branch with a single commit in `path` and return the
/// resulting commit SHA. The new branch has no common history with any
/// existing branch in the repo, so its tip and `main`'s share no ancestor.
/// Its tree holds one file, `<branch_name>.txt`.
///
/// After this call, `path` is left on the newly created orphan branch, with
/// the previous branch's tracked files removed from the working tree.
pub fn orphan_commit(path: &Path, branch_name: &str, message: &str) -> String {
    git(path, &["checkout", "--orphan", branch_name]);
    // Remove the previous branch's files from the index and the working tree,
    // so neither the commit nor a later `git add` picks them up.
    git(path, &["rm", "-rfq", "--ignore-unmatch", "."]);
    write(&path.join(format!("{branch_name}.txt")), branch_name);
    git(path, &["add", "."]);
    git(path, &["commit", "-m", message]);
    head(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rev(path: &Path, rev: &str) -> String {
        git(path, &["rev-parse", rev]).trim().to_string()
    }

    fn commit_count(path: &Path, range: &str) -> String {
        git(path, &["rev-list", "--count", range])
            .trim()
            .to_string()
    }

    #[test]
    fn scratch_clone_is_level_with_the_remote_and_tracks_it() {
        let (remote, clone) = scratch_remote_and_clone();
        assert_eq!(rev(clone.path(), "HEAD"), rev(remote.path(), "main"));
        assert_eq!(rev(clone.path(), "HEAD"), rev(clone.path(), "origin/main"));
        assert_eq!(
            git(clone.path(), &["rev-parse", "--abbrev-ref", "@{upstream}"]).trim(),
            "origin/main"
        );
    }

    #[test]
    fn behind_by_one_clone_fast_forwards_to_the_remote_on_pull() {
        let (remote, clone) = scratch_remote_and_clone_behind_by_one();
        assert_eq!(commit_count(remote.path(), "main"), "2");
        assert_eq!(rev(clone.path(), "HEAD"), rev(remote.path(), "main~1"));
        // The remote-tracking ref only sees the advance after a fetch.
        assert_eq!(commit_count(clone.path(), "HEAD..origin/main"), "0");
        git(clone.path(), &["fetch", "-q"]);
        assert_eq!(commit_count(clone.path(), "HEAD..origin/main"), "1");
        git(clone.path(), &["pull", "-q", "--ff-only"]);
        assert_eq!(rev(clone.path(), "HEAD"), rev(remote.path(), "main"));
    }

    #[test]
    fn push_sibling_commit_returns_the_remote_tip_and_repeats() {
        let (remote, clone) = scratch_remote_and_clone();
        let first = push_sibling_commit(remote.path(), "advance");
        assert_eq!(first, rev(remote.path(), "main"));
        assert_eq!(
            git(remote.path(), &["log", "-1", "--format=%s", "main"]).trim(),
            "advance"
        );
        let second = push_sibling_commit(remote.path(), "advance");
        assert_eq!(second, rev(remote.path(), "main"));
        assert_eq!(rev(remote.path(), "main~1"), first);
        git(clone.path(), &["fetch", "-q"]);
        assert_eq!(commit_count(clone.path(), "HEAD..origin/main"), "2");
    }

    #[test]
    #[should_panic(expected = "git-fixture: cannot write")]
    fn push_sibling_commit_names_an_unwritable_message_file() {
        let (remote, _clone) = scratch_remote_and_clone();
        push_sibling_commit(remote.path(), "fix: a/b");
    }

    #[test]
    fn local_commit_commits_without_pushing_and_repeats() {
        let (remote, clone) = scratch_remote_and_clone();
        let before = head(clone.path());
        local_commit(clone.path(), "a.txt", "one");
        local_commit(clone.path(), "a.txt", "two");
        assert_eq!(commit_count(clone.path(), &format!("{before}..HEAD")), "2");
        assert_eq!(
            git(clone.path(), &["log", "-1", "--format=%s"]).trim(),
            "two"
        );
        assert_eq!(git(clone.path(), &["show", "HEAD~1:a.txt"]), "content");
        assert_eq!(rev(remote.path(), "main"), before);
    }

    #[test]
    fn orphan_commit_holds_only_its_own_file_and_shares_no_ancestor() {
        let (_remote, clone) = scratch_remote_and_clone();
        let main = head(clone.path());
        let a = orphan_commit(clone.path(), "orphan-a", "orphan a");
        let b = orphan_commit(clone.path(), "orphan-b", "orphan b");
        assert_eq!(head(clone.path()), b);
        assert_eq!(
            git(clone.path(), &["symbolic-ref", "HEAD"]).trim(),
            "refs/heads/orphan-b"
        );
        for (tip, file) in [(&a, "orphan-a.txt"), (&b, "orphan-b.txt")] {
            assert_eq!(
                git(clone.path(), &["ls-tree", "--name-only", tip]).trim(),
                file
            );
            assert_eq!(commit_count(clone.path(), tip), "1");
        }
        for (x, y) in [(&main, &a), (&main, &b), (&a, &b)] {
            assert!(!crate::try_git(clone.path(), &["merge-base", x, y]));
        }
        assert!(
            git(clone.path(), &["status", "--porcelain"]).is_empty(),
            "no file from an earlier branch may linger in the working tree"
        );
        git(clone.path(), &["checkout", "-q", "main"]);
        assert_eq!(head(clone.path()), main);
    }
}
