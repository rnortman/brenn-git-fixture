# TODOs

Deferred work, one `## \`slug\`` entry per item, joined by slug to the
`TODO(slug)` comment where the work belongs. This file ships publicly.

## `scrub-ci-tree-scan`

The `scrub` job in `.github/workflows/ci.yml` is a hand-written gitleaks tree
scan: it downloads gitleaks at a version and sha256 pinned in the workflow,
runs `gitleaks dir .` against `.gitleaks.toml`, and drops
`ephemeral-reference` findings with `jq`. That is what `brenn-scrub tree`
does, but nothing ties the workflow's gitleaks pin to the version
`brenn-scrub` enforces for the local hooks, so a pin bump in `brenn-scrub`
leaves CI scanning with the old release. Replace the job with brenn-scrub's
reusable `tree-scan.yml` workflow at a release tag, and add
`brenn-scrub init --check` to `make check`, once that workflow is published.
