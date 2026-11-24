# Ricochet-Robots

Online multiplayer version of the Ricochet Robots board game, originally a
college project. A rewrite to a Rust backend and Svelte client is in
progress; see `docs/rewrite-plan.md`.

## Development setup

```bash
git clone --recurse-submodules https://github.com/basioli-k/Ricochet-Robots
pipx install pre-commit && pre-commit install   # gitleaks on every commit
```

`.agents/standards` is a private submodule with the shared engineering rules
(imported by `CLAUDE.md`). Without access to it the app still works; skip
`--recurse-submodules`, and ask the repository owner for access if you need
the rules.

### Legacy PHP app (`legacy/`)

1. Serve `legacy/` as the document root, and set `RR_DB_DSN`, `RR_DB_USER`
   and `RR_DB_PASSWORD` in the web server (see `.env.example` for Apache
   and PHP-FPM examples).
2. Export the same variables in your shell (the web server settings don't
   reach the command line), then create tables and demo accounts once:
   ```bash
   [ -f .env ] || cp .env.example .env   # then fill in the values
   set -a; . ./.env; set +a
   php legacy/app/database/create_tables.php && php legacy/app/database/seed_tables.php
   ```
   The seed prints the generated passwords. A second run stops at
   `create_tables.php` because the table already exists.
