@.agents/standards/standards/engineering.md

# Ricochet Robots

Online multiplayer version of the Ricochet Robots board game.

## State of the code

The legacy PHP + jQuery app has been replaced and removed (its board data
survives in `tools/legacy_board.php` for `tools/board_oracle.py`). Follow
`docs/rewrite-plan.md` for the architecture; it also records the audit of
the old app.

## Target stack

- Backend: Rust workspace (`crates/domain`, `crates/storage`,
  `crates/server`), axum + tokio, SQLx.
- Frontend: Svelte 5 + TypeScript (strict) + Vite in `web/`.
- The server is authoritative for all game rules; the client only renders
  state and sends intents.
- `crates/domain` stays pure: no I/O, async or framework types.
- Database: SQLite.
- User-facing text in English and Croatian, via the translation module.
- One fixed board for now; any new board must pass `BoardValidator`.
- Sign-in is Google (OpenID Connect) or guest only; no passwords or email are
  stored, and from Google only its `sub` is kept.
- Guests can play without an account; any game with a guest is unranked.

## Commands

Run before every commit (CI runs the same):

- Rust: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --locked`
- If proptest writes a `*.proptest-regressions` file for a real failure,
  commit it with the fix: proptest replays those cases first. Delete ones
  produced by deliberate mutation checks.
- Web, from `web/`: `npm run lint`, `npm run typecheck`, `npm test`,
  `npm run build`, and `npm run e2e` for UI changes
- Tools: `python3 -I -m unittest discover -s tools -p 'test_*.py'`

## Dependencies

Only adopt releases published at least 14 days ago: fresh releases are where
compromised publishes usually get caught. This covers crates and npm
packages (checked in CI by `tools/check_dependency_age.py`) and CI itself:
actions are pinned to the commit SHA of an old-enough release, and tools CI
installs are pinned to an old-enough version (checked by hand).
Container images follow the same rule by digest (checked by hand): each
pinned digest, not just its version tag, is at least 14 days old; the
Dockerfile comment records each digest's date.
Exceptions go in `.dependency-age-allow` with a reason: a known
vulnerability outranks the rule (take the fix, cite the advisory), and the
owner may approve a specific case (cite the approval). See README for the
commands.

## Secrets

Never commit credentials or personal data, including in seeds and tests.
Read config from environment variables and document names in
`.env.example`. See `docs/rewrite-plan.md` section 6.

## Pull requests

Every change reaches `main` through a pull request, including
`chore: update agent standards` bumps. Review follows the shared workflow
(`.agents/standards/standards/pull-requests.md`). Project specifics:

- Standards bumps and small leftovers from the previous PR's review may ride
  along in the current PR as their own commit, overriding "one topic per
  branch". Name leftovers as such in the commit message and PR description.
- After approval, push the branch to `main` as a fast-forward (rebase onto
  `main` first if needed).
