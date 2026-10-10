# brenn-git-fixture

Hermetic git spawning for test fixtures, with a repo-escape canary.

`Command::new("git").current_dir(dir)` does not isolate a test from the
repository it runs in. git's environment variables — `GIT_DIR`,
`GIT_INDEX_FILE`, `GIT_WORK_TREE`, the `GIT_CONFIG_*` family — override
`current_dir`, and a `git commit` from a linked worktree exports `GIT_DIR` and
`GIT_INDEX_FILE` into its pre-commit hook. A test suite run from that hook
inherits them, and every fixture `git commit` lands in the real repository
instead of the test's tempdir.

This crate makes isolation a property of construction:

- **Hermetic fixture spawns.** `git`, `try_git` and `hermetic` strip every
  `GIT_*` variable (inherited or set on the `Command`), pin global and system
  config to `/dev/null`, stop upward repo discovery at the temp root, and set
  a fixed fixture identity.
- **A non-hermetic canary.** Every helper that roots a fixture repo ends in
  `assert_repo_is`, which spawns git the way production code does — inherited
  environment, `current_dir` only — and panics if the git dir or the index
  resolves outside the fixture. It catches both a fixture that escaped and an
  environment that would redirect the code under test.

## Use

Dev-only by convention: depend on it from `[dev-dependencies]`.

```toml
[dev-dependencies]
brenn-git-fixture = "0.1"
tempfile = "3"
```

In a test:

```rust
use brenn_git_fixture::{git, seed_repo};

let dir = tempfile::tempdir().unwrap();
seed_repo(dir.path()); // init on `main`, one commit, canary checked
let head = git(dir.path(), &["rev-parse", "HEAD"]);
assert!(!head.trim().is_empty());
```

`cargo test` runs this example as a doctest.

The system `git` must be on `PATH`. Every helper panics with git's stderr and
stdout on failure: a fixture step that did not do what it said is a broken
test.

## API

| Item | What it does |
|---|---|
| `hermetic(&mut Command)` | Strip `GIT_*`, pin config sources and identity on a `Command` you build yourself. |
| `git(dir, args) -> String` | Run `git <args>` hermetically in `dir`; stdout, or panic. |
| `try_git(dir, args) -> bool` | Same, reporting success instead of panicking — for a step meant to fail. |
| `init_repo(dir)` | `git init -b main` with a local identity, then the canary. |
| `init_bare_repo(dir)` | Bare repo on `main`, then the canary. |
| `clone_repo(src, dest)` | Hermetic clone with a local identity, then the canary. |
| `seed_repo(dir)` | `init_repo` plus one commit, so `HEAD` exists. |
| `add_bare_origin(dir) -> TempDir` | A bare `origin` in a fresh tempdir with `main` pushed and tracked; keep the `TempDir` alive. |
| `assert_repo_is(dir)` | The canary. |
| `scratch::*` | Bare-remote-plus-clone pairs (`scratch_remote_and_clone`, `…_behind_by_one`), `push_sibling_commit`, `local_commit`, `orphan_commit`, `head`. |

Fixture commits are authored by `alice <a@example.com>`. Finding that identity
in a real repository's config, reflog or history means some git spawn escaped
its fixture.

## Development

```sh
make setup-hooks   # point git at .githooks; prints what to install if a hook's tool is missing
make check         # msrv, fmt, clippy, test — what CI and pre-commit run
```

## License

Apache-2.0. See `LICENSE` and `NOTICE`.
