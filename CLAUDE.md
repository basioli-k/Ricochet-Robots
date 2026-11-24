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

## Secrets

Never commit credentials or personal data, including in seeds and tests.
Read config from environment variables and document names in
`.env.example`. See `docs/rewrite-plan.md` section 6.
