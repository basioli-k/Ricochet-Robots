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

### Rust workspace (`crates/`)

Needs `rustup`.

```bash
rustup toolchain install   # once: installs the version in rust-toolchain.toml
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

### Web client (`web/`)

Needs Node.js 22.13+ or 24+.

```bash
cd web
npm ci --ignore-scripts   # as in CI
npm run dev         # local dev server
npm run lint
npm run typecheck
npm test
npm run build
```

### Adding or updating dependencies

Only releases at least 14 days old are allowed (see `CLAUDE.md`). Check with
`python3 -I tools/check_dependency_age.py`; CI runs it, `cargo audit` and
`npm audit` on every pull request and weekly.

- npm: `npm install --save-dev --before=YYYY-MM-DD pkg@major`, with the date
  14 days ago.
- Cargo: add with an explicit old-enough version, `cargo add name@1.2.3`;
  `cargo add name` writes the newest release as the minimum requirement, and
  `cargo update --precise` can't go below a requirement. For transitive
  crates the check flags, `cargo update -p name@new --precise older` works
  as long as no `Cargo.toml` requirement forbids the older version.
- Security fixes newer than that go in `.dependency-age-allow` with the
  advisory ID; the check notes entries that are no longer needed.
- GitHub Actions are pinned to the commit SHA of a release at least 14 days
  old, with the version in a comment.

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

Make `.env` unwritable by the web server user, or delete it after seeding:
legacy endpoints can read or empty any file the server can both read and
write, even outside the docroot (audit finding S1).

The game keeps its state in `legacy/**/*.log`. They change while the app
runs; don't commit those changes. To reset a stuck game, empty them with
`cd legacy && python3 clear_log_files.py` (it only works from `legacy/`).
