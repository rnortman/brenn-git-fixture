# Changelog

Notable changes to brenn-git-fixture. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-10-10

First public release. The crate was developed inside the
[brenn](https://github.com/rnortman/brenn) repository as `git-fixture`; this
repository starts from an import of that source.

brenn-git-fixture builds throwaway git repositories for Rust tests and runs
git against them *hermetically*: the developer's own git configuration and
environment cannot leak in, and a built-in safety check (the "repo-escape
canary") fails the test if git ever resolves to a repository outside the
fixture.

### Added

- `hermetic`, `git`, `try_git` — run git with every `GIT_*` variable
  stripped, global and system config pinned to `/dev/null`, a
  `GIT_CEILING_DIRECTORIES` stop at the temp root, and a fixed fixture
  identity.
- `init_repo`, `init_bare_repo`, `clone_repo`, `seed_repo`,
  `add_bare_origin` — root fixture repositories on `main`, each ending in the
  repo-escape canary.
- `assert_repo_is` — the canary: a deliberately non-hermetic check that git,
  spawned the way production code spawns it, resolves both its git dir and its
  index inside the fixture.
- `scratch` module — bare-remote-plus-clone pairs and commit helpers for tests that
  exercise a real pull.

### Changed from brenn's `git-fixture`

- `scratch::orphan_commit` commits a tree holding only `<branch>.txt` and
  removes the previous branch's tracked files from the working tree; it used
  to carry them into the orphan commit.
- `scratch::push_sibling_commit` and `scratch::local_commit` make an empty
  commit when a repeat call changes no file, instead of panicking.
- `git` panics report git's stdout as well as its stderr.
