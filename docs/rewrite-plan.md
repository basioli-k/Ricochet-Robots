# Audit and rewrite plan

Status: decisions agreed (section 5). Measured against `.agents/standards/standards/engineering.md`.

## 1. What exists today

A PHP 7 MVC-ish app (`index.php` router, `controller/`, `model/`, `view/`)
for accounts and stats, plus a multiplayer game built from:

- jQuery in the browser (`js/gameplay.js`, ~670 lines), which holds **all**
  game logic: robot movement, bidding order, win detection, scoring, round
  flow.
- 13 PHP scripts in `app/` that read and write plain `.log` files
  (`potezi.log`, `licitacija.log`, `timer.log`, ...) and long-poll them with
  `usleep(10ms)` loops. These files are the whole "game server".
- MySQL for players only (hard-coded connection in
  `app/database/database.class.php`).

There is one global game per deployment, no tests, no build, no lint, and
identifiers, comments and messages mix Croatian and English.

## 2. Audit findings

Severity: **C** critical, **H** high, **M** medium, **L** low.

### Security

| # | Sev | Where | Problem |
|---|-----|-------|---------|
| S1 | C | `app/dajLog.php`, `app/ocistiLog.php` | `filename`/`filenames` come straight from the query string. With `../` paths, anyone can read or empty any file the web server can both read and write (the scripts check both), e.g. PHP sources holding the DB credentials on a typical shared-host setup. No session check. |
| S2 | C | `app/database/database.class.php`, `seed_tables.php` | Database credentials and real people's emails and plaintext passwords were committed. **Done:** redacted from all history with `git filter-repo`; credentials now come from the environment; gitleaks runs in pre-commit and CI (see section 6). Still treat the old values as leaked and change them. |
| S3 | H | `app/updatajRezultate.php`, `app/updateBaze.php` | The client reports its own score (`?username=X&bodovi=N`), and anyone can call `updateBaze.php` any number of times. Points and stats can be forged for any player. |
| S4 | H | `app/saljiPoruku.php`, `gameplay.js:588` | Stored XSS: the chat message is stored as HTML and inserted with `.append()`. The sender name `ime` comes from the request, so players can also impersonate each other. |
| S5 | H | `controller/statsController.php`, `Model::order_by` | SQL injection: `btn_stats` from POST goes into `ORDER BY` unescaped. |
| S6 | M | `app/*.php` | No endpoint checks the session; identity comes from a non-HttpOnly `username` cookie read by JS. |
| S7 | M | `index.php` | `rt` picks the controller file and method (`$con->$action()`), so any public method is callable; the logged-out regex `^(login\|register)(.)*$` is loose. |
| S8 | M | `registerController.php` | Confirmation token from `rand()` (not a CSPRNG), never expires or gets cleared; no CSRF protection on any form. |
| S9 | L | `loginController.php` | The whole `Player` row, password hash included, lives in `$_SESSION`. DB error text is echoed to users via `exit()`. |

### Correctness

| # | Sev | Where | Problem |
|---|-----|-------|---------|
| B1 | H | `gameplay.js:krajIgre` | Every client runs end-of-game, so `updateBaze.php` runs once per player and stats get counted N times. |
| B2 | H | all `.log` writers | Read-modify-write without locks (`player_count.log`, `rezultati.log`, `usernames.log`): concurrent requests lose updates. |
| B3 | H | `app/cek*.php` | Long-poll loops have no timeout and spin every 10 ms, holding a PHP worker per waiting client; a few players can exhaust the pool. |
| B4 | H | `index.php` | A non-empty `timer.log` blocks everyone from starting a game; a crashed or abandoned game blocks the site until files are cleared by hand. |
| B5 | M | `jesamPrvi.php` | "Am I the host" means "`usernames.log` has exactly one name": racy, and wrong after a rejoin. |
| B6 | M | `robot_movement.js` | Walls are found by comparing computed CSS to `"5px solid rgb(165, 42, 42)"`; robots are identified by background colour. Breaks on any style or browser change. A wall stops movement only if it is encoded on the current cell's side; nothing guarantees the neighbour has the mirrored wall. |
| B7 | M | `Model::all()` | Calls `generateAttributeList()` without `static::`, a fatal error if ever used. |
| B8 | M | `registerController::newUser` | Renders the view, then calls `$this->index('...')`, which ignores the argument and renders again. |
| B9 | M | `gameplay.js` | `async: false` XHR throughout (freezes UI, deprecated); `while(!confirm())` loop; sync AJAX plus navigation in `onbeforeunload`. |
| B10 | L | `gameplay.js:dobroRijeseno` | Mutates the shared target (`"purple"` to `"#800080"`) as a side effect of a check. |
| B11 | L | `view/gameplay.php` | Loads jQuery twice (3.6.0 and 3.3.1) and `utils.js` twice; `login.php` has malformed `<b/>`. |
| B12 | L | `dajLog.php` | Magic `17` (target count) duplicated from the target list, which is itself duplicated in `game.class.php` and `jesamPrvi.php`. |

### Design and style (against the standards)

- **Domain logic is not pure.** Rules live in the browser and are tangled with
  DOM queries; `Field::draw_field` and `Game::draw_board` echo HTML and read
  `$_SESSION`.
- **No server authority.** The server cannot validate a move or a solution,
  because it has no model of the board.
- **No abstractions or injection.** Static `DB::getConnection()` singleton,
  file paths hard-coded in each script, `sendJSONandExit` and the file-check
  block copy-pasted into 13 files.
- **Only one fixed board** (a 16x16 literal in `gameplayController.php`);
  robot positions never randomise.
- **Language:** mixed Croatian and English identifiers and comments;
  user-facing strings hard-coded in views and JS.
- **No tests, lint, types or CI**; runtime `.log` files and
  `clear_log_files.py` are committed.

Conclusion: nothing on the backend is worth porting line by line. Keep the
game design, the board layout (as a test fixture) and the page flow; rewrite
the rest.

## 3. Target architecture

```
ricochet-robots/
├── Cargo.toml                 # workspace
├── crates/
│   ├── domain/                # pure rules: no I/O, no async, no serde_json
│   ├── storage/               # repository traits + SQLx and in-memory impls
│   └── server/                # axum HTTP + WebSocket, rooms, auth, mailer
├── web/                       # TypeScript (strict) + Vite client
├── migrations/                # SQLx migrations
└── .agents/standards/         # submodule
```

### `domain` crate (pure, fully unit-tested)

- Value types: `Position`, `Direction`, `RobotColor`, `Symbol`, `Target`.
- `Board`: walls stored as per-cell bitflags; the builder sets both sides of
  a wall so B6 cannot recur. For now there is one board: the current fixed
  layout, ported as data. Random quadrant boards come later, once boards can
  be verified (below).
- `BoardValidator`: checks a board is legal (closed outer walls, blocked
  centre, mirrored walls, one cell per target, every target reachable by its
  robot from the start position). Any new board must pass it before use.
- `Board::slide(robots, robot, dir) -> Position`: the one movement function,
  used by server validation, client preview and the solver.
- `Round` state machine: `Bidding` -> `Demonstrating { queue }` ->
  `Resolved`. Bids ordered by fewest moves, then earliest; a player may lower
  their bid. Failed demonstrations reset robots and pass to the next bidder.
- `Game`: shuffled target deck, scores, game-over.
- Injected `Clock` and `Rng` traits so timers and shuffles are
  deterministic in tests.
- BFS `Solver`, used by `BoardValidator` and later for hints.

### `storage` crate

- `PlayerRepository` and `GameResultRepository` traits, each with an SQLx
  implementation and an in-memory one for tests.
- Store finished games (`games`, `game_players`) and compute stats with
  queries instead of incrementing counters, so B1-style double counting
  can't corrupt data, and the leaderboard sort is a closed enum (fixes S5).
- SQLite (one file, easy to deploy); the trait keeps other databases open.

### `server` crate

- `axum` + `tokio`, `tracing`, `thiserror` errors mapped to HTTP responses
  without leaking internals.
- Auth (sign-up, email confirmation, login, logout; replaces the PHP
  controllers): argon2id hashes, `tower-sessions` cookie (HttpOnly, SameSite),
  CSRF token on forms, email confirmation tokens from a CSPRNG with an
  expiry, sent through a `Mailer` trait (SMTP via `lettre`; a logging mailer
  for development).
- Rooms: `RoomManager` creates any number of rooms (fixes B4 and B5). Each
  room is a tokio task that owns a `domain::Game` and takes commands over an
  mpsc channel, so there is no shared mutable state (fixes B2). Events go out
  over a broadcast channel.
- WebSocket protocol, server-authoritative. The player is taken from the
  session, never from the payload (fixes S3, S4, S6):
  - client -> server: `Join`, `Start`, `Bid { moves }`, `Move { robot, dir }`,
    `Chat { text }`, `Leave`
  - server -> client: `Snapshot`, `BidPlaced`, `BiddingClosesAt { deadline }`,
    `DemonstrationStarted { player, moves }`, `RobotMoved`, `RoundResolved`,
    `Chat`, `GameOver { scores }`
- Timers run on the server; clients only display the deadline.
- Disconnects detected by WebSocket close and heartbeat. Results persisted
  exactly once, when the room's game ends.
- Protocol types exported to TypeScript with `ts-rs` so client and server
  can't drift.
- All configuration and secrets from environment variables (`.env` locally,
  git-ignored; `.env.example` documents the names).

### `web` client

- Svelte 5 + TypeScript `strict`, built with Vite as a static SPA that the
  Rust server serves. ESLint, `svelte-check`, Vitest, Playwright.
- Board drawn from server state (CSS grid or SVG); walls and robots come from
  data, never from computed styles. Keyboard (WASD and arrows) and click
  controls, plus the robot-slide animation from `todos.txt`.
- All user-facing strings in one translation module, English and Croatian. Chat rendered as text, never HTML.
- Pages: login, register, lobby (room list), game, leaderboard.

## 4. Delivery plan

Each step is a branch named per the standards (`<type>/<kebab>`), split into
commits of at most ~200 lines of logic, each shipping its tests.

| Phase | Branch | Content |
|-------|--------|---------|
| 0 | `chore/agent-standards` | Standards submodule, `CLAUDE.md`, this plan, history redaction, secret scanning. Done. |
| 1 | `chore/workspace-skeleton` | Cargo workspace, `web/` Svelte + Vite scaffold, CI (`cargo fmt --check`, `clippy -D warnings`, `cargo test`, `npm run lint`/`typecheck`/`test`). Move PHP to `legacy/`; untrack `.log` files. |
| 2 | `feat/domain-board` | Value types, `Board` with mirrored walls, `slide`, the current board as fixture; property tests (a robot never crosses a wall; sliding twice in one direction is a no-op). |
| 3 | `feat/domain-round` | Targets, bidding order, `Round` state machine, `Game`, scoring, injected `Clock`/`Rng`. Table tests for each bidding rule. |
| 4 | `feat/domain-board-validation` | `Solver` and `BoardValidator`; test that the fixed board passes and that broken boards (open edge, one-sided wall, unreachable target) fail. |
| 5 | `feat/storage` | Traits, in-memory impls, SQLx SQLite impl, migrations, repository tests run against both impls. |
| 6 | `feat/server-auth` | Register, confirm, login, logout, session, CSRF, `Mailer`; HTTP integration tests. |
| 7 | `feat/server-rooms` | `RoomManager`, room actor, WebSocket protocol, `ts-rs` export; tests drive two fake clients through a full round. |
| 8 | `feat/web-auth-lobby` | Login, register, lobby, leaderboard pages; i18n module. |
| 9 | `feat/web-game` | Board rendering, controls, animation, bidding UI, chat; Playwright e2e with two browser contexts playing a round. |
| 10 | `chore/remove-legacy` | Delete `legacy/`, `clear_log_files.py`; Dockerfile and README with run and test commands. |
| later | `feat/domain-quadrants` | Quadrant set and rotation, random boards, each checked by `BoardValidator`. |
| later | `chore/hosting` | Pick a host (investigation pending) and add deployment. |

Phases 2-5 have no dependency on the web client and can land first; the PHP
app keeps working until phase 10.

## 5. Decisions

| Topic | Decision |
|-------|----------|
| Database | SQLite |
| Client | Svelte 5 + TypeScript (strict) |
| UI language | English and Croatian |
| Board | The single fixed board for now; random quadrant boards only after `BoardValidator` exists |
| Sign-up and email confirmation | Kept, reimplemented in Rust |
| Leaked data | Purged from git history; secret scanning added |
| Hosting | Undecided; investigate later. The Dockerfile in phase 10 stays host-agnostic. |

## 6. Secrets policy

- No credentials, tokens, personal data or real user accounts in the
  repository, including tests and seed data. Seeds create demo accounts on
  `example.invalid` with generated passwords.
- Configuration comes from environment variables; `.env` is git-ignored and
  `.env.example` lists the names with empty or dummy values.
- `.gitleaks.toml` extends the default gitleaks rules with patterns this repo
  has leaked before (inline PDO credentials, literal passwords in seeds,
  database URLs with passwords).
- Enforced twice: the gitleaks pre-commit hook (`pre-commit install`) and the
  `Secret scan` GitHub Actions workflow on every push and pull request.
- If something leaks anyway: rotate it first, then redact history with
  `git filter-repo --replace-text` and force-push.
