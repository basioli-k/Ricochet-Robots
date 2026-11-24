@.agents/standards/standards/engineering.md

# Ricochet Robots

Online multiplayer version of the Ricochet Robots board game.

## State of the code

The current code is a legacy PHP + jQuery app being replaced. Follow
`docs/rewrite-plan.md` for the target architecture and phase order; don't
extend the PHP code beyond fixes needed to keep it running.

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
- Guests can play without an account; any game with a guest is unranked.

## Commands

Run before every commit (CI runs the same):

- Rust: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --locked`
- Web, from `web/`: `npm run lint`, `npm run typecheck`, `npm test`,
  `npm run build`

## Secrets

Never commit credentials or personal data, including in seeds and tests.
Read config from environment variables and document names in
`.env.example`. See `docs/rewrite-plan.md` section 6.

## Pull requests

Every change reaches `main` through a pull request, including
`chore: update agent standards` bumps. Two kinds of change may ride along
in the current PR as their own commit, overriding "one topic per branch":
standards bumps, and small leftovers from the previous PR's review (named
as such in the commit message and PR description).

1. Push a `<type>/<kebab>` branch and open a PR against `main`.
2. Review it with three independent reviewer subagents, each with its own
   focus: correctness and security; these standards and the rewrite plan;
   tests, CI and docs. Each returns `APPROVE` or `REQUEST_CHANGES` with
   findings marked blocking or nit.
3. Fix every blocking finding; fix nits when cheap, otherwise note why not.
   Then re-review with all three.
4. On consensus (all three `APPROVE`), record the review outcome in the PR
   description, then push the branch to `main` as a fast-forward (rebase
   onto `main` first if needed).
5. If there is no consensus after three review rounds, stop and ask the
   owner.
