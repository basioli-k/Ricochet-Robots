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
`--recurse-submodules`.

### Legacy PHP app

1. Set `RR_DB_DSN`, `RR_DB_USER` and `RR_DB_PASSWORD` in the web server
   (see `.env.example` for Apache and PHP-FPM examples).
2. Create tables and demo accounts from the command line:
   `php app/database/create_tables.php && php app/database/seed_tables.php`
   (the seed prints the generated passwords).
