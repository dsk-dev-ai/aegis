# Development Workflow

This document is the contract for how work lands in Aegis. It keeps history
clean, reviewable, and easy to revert, so any past state is recoverable.

## Branching model

- `main` is always the integration trunk and is protected by convention: no
  direct commits, only merges from short-lived branches.
- One branch per change, named by type:
  - `feat/<area>-<summary>` — new functionality
  - `fix/<area>-<summary>` — bug fixes
  - `docs/<summary>` — documentation only
  - `chore/<summary>` — tooling, CI, deps
  - `refactor/<area>-<summary>` — no behaviour change
- Keep branches short-lived and scoped to a single concern.

## Commit & PR conventions

- Conventional Commits for messages/titles:
  `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `build:`, `ci:`.
- PR title becomes the squashed commit subject, so make it descriptive.
- PR body must include **Summary** and **Verification** sections.

## Definition of done (local gate)

A change is mergeable only when it passes locally:

```bash
# Rust workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Flutter app
cd apps/mobile
flutter analyze
flutter test
```

For relay-touching changes, also run the end-to-end test:

```bash
cargo run -p aegis-server &
AEGIS_RELAY_URL=ws://127.0.0.1:8080/v1/ws flutter test test/relay_e2e_test.dart
```

## Merge policy

- **Local green is the gate.** Once the definition of done passes locally, the
  PR is merged immediately; CI is informational and does not block.
- Merge method: **squash and merge** — one tidy commit per PR on `main`.
- Delete the branch after merge.

```bash
gh pr create --base main --head <branch> --title "<type>: <summary>" --body "..."
gh pr merge <n> --squash --delete-branch
git checkout main && git pull
```

## Releases

- Tag releases on `main` as `vMAJOR.MINOR.PATCH` (e.g. `v1.0.0`).
- Each tag has notes generated from the squashed PR titles.
- `0.x` = pre-1.0 (API/format may change); `1.0.0` = first stable release.

## Recoverability

- Every change is one revertible squashed commit on `main`.
- Never rewrite published history on `main` (no force-push).
- Feature work stays on branches until it is locally verified.
