# Ricochet-Robots

Online multiplayer version of the Ricochet Robots board game, originally a
college project. Rewritten from PHP and jQuery to a Rust server and a
Svelte client; see `docs/rewrite-plan.md` for the design and its history.

## Development setup

```bash
git clone --recurse-submodules https://github.com/basioli-k/Ricochet-Robots
pipx install pre-commit && pre-commit install   # gitleaks on every commit
```

`.agents/standards` is a private submodule with the shared engineering rules
(imported by `CLAUDE.md`). Without access to it the app still works; skip
`--recurse-submodules`, and ask the repository owner for access if you need
the rules.

### Running the whole app

```bash
(cd web && npm ci --ignore-scripts && npm run build)   # writes web/dist
cargo run -p server                                    # http://localhost:8080
```

The server serves the built client from `RR_WEB_DIR` (default `web/dist`)
next to the API; settings are listed in `.env.example`. For development,
run `npm run dev` in `web/` instead and open http://localhost:5173.

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
npm run dev         # local dev server (proxies /api to the Rust server on :8080)
npm run lint
npm run typecheck
npm test
npm run build
npm run e2e         # Playwright, two guests playing a round (waits out the 60 s bidding window)
```

`npm run e2e` starts the Rust server and the dev server itself (or reuses
running ones locally) and needs a Chromium for the pinned
`@playwright/test`: `npx playwright install chromium` once.

### Adding or updating dependencies

Only releases at least 14 days old are allowed (see `CLAUDE.md`). Check with
`python3 -I tools/check_dependency_age.py`; CI runs it, `cargo audit` and
`npm audit` on every pull request and weekly.

- npm: `npm install --save-dev --before=YYYY-MM-DD pkg@major`, with the date
  14 days ago.
- Cargo: `cargo add name@1.2.3` with an old-enough version (plain
  `cargo add name` makes the newest release the minimum, and nothing can be
  locked below a minimum). It still locks the newest compatible release, so
  follow with `cargo update -p name --precise 1.2.3`; do the same for any
  transitive crate the check flags.
- Exceptions go in `.dependency-age-allow` with a reason: a security fix
  (cite the advisory) or an owner-approved case (cite the approval); the
  check notes entries that are no longer needed.
- GitHub Actions are pinned to the commit SHA of a release at least 14 days
  old, with the version in a comment.
