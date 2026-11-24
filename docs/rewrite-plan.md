# Audit and rewrite plan

Status: decisions agreed (section 5). Measured against
`.agents/standards/standards/engineering.md`.

## 1. What exists today

Paths in sections 1 and 2 are as audited; since phase 1 they live under
`legacy/`.

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
| S1 | C | `app/dajLog.php`, `app/ocistiLog.php` | `filename`/`filenames` come straight from the query string. With `../` paths, anyone can read or empty any file the web server can both read and write (the scripts check both), e.g. the game's `.log` state files or config. No session check. |
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
| B1 | H | `gameplay.js:krajIgre` | Every client runs end-of-game, so `updateBaze.php` can run once per player before the logs are cleared: stats are counted between 1 and N times (racy). |
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
├── web/                       # Svelte 5 + TypeScript (strict) + Vite client
├── migrations/                # SQLx migrations
└── .agents/standards/         # submodule
```

### `domain` crate (pure, fully unit-tested)

- Value types: `Position`, `Direction`, `RobotColor`, `Symbol`, `Target`,
  and an opaque `PlayerId`. The domain doesn't know whether a player is a
  guest or has an account.
- `Board`: walls stored as per-cell bitflags; the builder sets both sides of
  a wall so B6 cannot recur. For now there is one board: the current fixed
  layout, ported as data. Random quadrant boards come later, once boards can
  be verified (below).
- Porting the fixed board (done in phase 2): the `new Field("....")`
  literal in `gameplayController.php` was ported by hand into
  `crates/domain/src/fixed_board.rs`, keeping the legacy wall encoding.
  `legacy/tools/board_oracle.py` simulates the legacy game on the PHP data
  to produce the parity tests' expected values. No wall was one-sided.
- Robots start at the legacy positions; random placement arrives with
  random boards.
- `BoardValidator`: checks what construction can't guarantee: the centre
  is blocked, all 17 targets are placed, robots start on free cells that
  hold no target (else a round could be won in zero moves), and
  every target is reachable by its robot from the start position. Any new
  board must pass it before use. (Since phase 2 the builder makes walls
  two-sided and targets one per cell, and the edge always stops movement.)
- `Board::slide(robots, robot, dir) -> Position`: the one movement function,
  used by server validation, client preview and the solver.
- `Round` state machine: `Bidding` -> `Demonstrating` (bidders in ranked
  order) -> `Solved` or `Unsolved`; a round nobody bids on is abandoned. Bids
  ordered by fewest moves, then earliest; a player may only lower their bid.
  Failed demonstrations reset robots and pass to the next bidder.
- A target the robots already complete when it comes up (robots carry over
  between rounds) goes to the bottom of the deck instead of being played;
  the game ends early if no remaining target is playable.
- `Game`: shuffled target deck, scores, game-over.
- Deterministic by construction: callers pass the time in as `Timestamp`
  values (the domain never reads a clock), and the target deck is ordered
  by an injected `Shuffle`. The server's `Clock` and `Rng` feed these.
- BFS `Solver`, used by `BoardValidator` and later for hints.

### `storage` crate

- `PlayerRepository` and `GameResultRepository` traits, each with an SQLx
  implementation and an in-memory one for tests.
- Store finished games (`games`, `game_players`) and compute stats with
  queries instead of incrementing counters, so B1-style double counting
  can't corrupt data, and the leaderboard sort is a closed enum (fixes S5).
- `game_players` keeps every participant so final scores stay complete:
  `account_id` for accounts, or `is_guest = true` with `guest_name` for
  guests. History is attributed only by `account_id`, never by name, so a
  later account with a guest's old nickname inherits nothing.
- `games.ranked` is set when the game is stored: true only if every player
  had an account and there were at least two. Stats and the leaderboard
  read ranked games only, so guests can't be used to farm wins (see
  Guests). The winner is the highest score; ties all count as wins.
- Fresh database: no data is imported from the legacy MySQL.
- SQLite (one file, easy to deploy); the trait keeps other databases open.

### `server` crate

- `axum` + `tokio`, `tracing`, `thiserror` errors mapped to HTTP responses
  without leaking internals.
- Dependencies injected through an `AppState` built in `main`:
  `Arc<dyn PlayerRepository>`, `Arc<dyn GameResultRepository>`,
  `Arc<dyn Mailer>`, `Arc<dyn Clock>`, `Arc<dyn Rng>`,
  `Arc<dyn RateLimiter>` (driven by the `Clock`). Tests build it with
  in-memory and fake implementations.
- Text: errors and events carry stable codes (e.g. `username_taken`), never
  prose; the client translates them. The only server-rendered text is email,
  from templates in English and Croatian, picked by the account's `locale`.
  That is the same field the client's language preference saves to, so they
  can't drift. Guests get no email; their language lives in the client.
- Auth (sign-up, email confirmation, login, logout; replaces the PHP
  controllers): argon2id hashes, `tower-sessions` cookie (HttpOnly, SameSite),
  CSRF token on every state-changing request (including `POST /guest`),
  session ID cycled on login, registration and guest start (prevents
  session fixation), expired sessions purged by the store's expiry task,
  email confirmation tokens from a CSPRNG with an expiry, sent through a
  `Mailer` trait (SMTP via `lettre`; a logging mailer for development).
- Guests: anyone can play without registering. `POST /guest` with a
  nickname starts a guest session (same cookie, no password or email),
  which expires after 24 hours of inactivity. The session holds
  `Identity::Account(id)` or `Identity::Guest { id, nickname }`; the room
  only sees a `PlayerId`, a display name and an `is_guest` flag.
  - `PlayerId` is a random UUID for both kinds, so a guest's ID can never
    equal an account's.
  - Names: one rule for usernames and nicknames, ASCII `[A-Za-z0-9]{3,15}` (no
    Unicode, so no lookalike letters). Usernames are unique ignoring case
    (`Alice` is rejected when `alice` exists). A nickname must be unique in
    the room and must not match a registered username, ignoring case; checked
    at guest start and again on every room join. Guests are always shown with
    a guest badge (`is_guest` in `Snapshot` and player lists), so a guest
    whose name is registered later still can't pass as that account.
  - Guests can create and join rooms, but games with any guest are
    unranked (see storage), so guests can't inflate anyone's stats.
- Abuse limits, all through `RateLimiter` and configurable (defaults: 5 guest
  starts and 5 room creations per IP per 10 minutes, at most 2 open rooms per
  session and per IP, 1 chat message per second per player, 10 bids or moves
  per second per player, at most 200 open rooms). Rooms with no connected
  players are closed after 10 minutes. The setting names are added to
  `.env.example` in the phase that introduces each limit. Behind a reverse
  proxy the client IP comes from a trusted `X-Forwarded-For` hop, configured
  with hosting.
- Rooms: `RoomManager` runs many concurrent rooms, up to the configured caps
  (fixes B4 and B5). Each room is a tokio task that owns a `domain::Game` and
  takes commands over an mpsc channel, so there is no shared mutable state
  (fixes B2). Events go out over a broadcast channel.
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
- All user-facing strings in one translation module, English and Croatian.
  Language comes from the saved preference, falling back to the browser
  language. Chat is rendered as text, never HTML.
- Pages: login (with "Play as guest"), register, lobby (room list), game,
  leaderboard.

## 4. Delivery plan

Each step is a branch named per the standards (`<type>/<kebab>`), split into
commits of at most ~200 lines of logic, each shipping its tests.

| Phase | Branch | Content |
|-------|--------|---------|
| 0 | `chore/agent-standards` | Standards submodule, `CLAUDE.md`, this plan, history redaction, secret scanning. Done in PR #5. |
| 1 | `chore/workspace-skeleton` | Cargo workspace, `web/` Svelte + Vite scaffold, CI (`cargo fmt --check`, `clippy -D warnings`, `cargo test`, `npm run lint`/`typecheck`/`test`). Move PHP to `legacy/` (its `.log` state files stay tracked: the app requires them to exist, and they go with `legacy/` in phase 10). |
| 2 | `feat/domain-board` | Value types, `Board` with mirrored walls, `slide`, the current board as fixture; property tests (a robot never crosses a wall; sliding twice in one direction is a no-op). |
| 3 | `feat/domain-round` | Targets, bidding order, `Round` state machine, `Game`, scoring, time passed in as values, injected `Shuffle`. A test for each bidding and round rule. |
| 4 | `feat/domain-board-validation` | `Solver` and `BoardValidator`; test that the fixed board passes and that broken boards (centre not blocked, missing target, robot on a blocked cell or on a target, unreachable target) fail. |
| 5 | `feat/storage` | Traits, in-memory impls, SQLx SQLite impl, migrations, repository tests run against both impls, including: a game with a guest stores the guest's score and is unranked; a one-player game is unranked; stats count ranked games only; ties count as wins. |
| 6a | `feat/server-auth` | `AppState`, register, confirm, login, logout, session (ID cycling, expiry purge), CSRF, `Mailer` with EN/HR email templates, account locale; HTTP integration tests, including case-insensitive username uniqueness and a new session ID after login. |
| 6b | `feat/server-guests` | Guest sessions and `RateLimiter`; tests: `RateLimiter` with a fake clock, the guest-start limit, name rule (length, non-ASCII rejected), clash with a username in any case, 24-hour expiry with a fake clock, CSRF on `POST /guest`. |
| 7a | `feat/server-rooms` | `RoomManager` and room actor over `domain::Game` (each game gets a `Shuffle` backed by the injected `Rng`), room caps and idle closing; tests drive commands through a full round without a network, and cover a duplicate nickname in a room, the room-creation limit, the per-session/IP and global room caps, and idle closing with a fake clock. |
| 7b | `feat/server-websocket` | WebSocket protocol, session-based identity, `ts-rs` export; tests with two WebSocket clients, one of them a guest who finishes a round and is flagged `is_guest`, plus the chat and bid/move rate limits. |
| 8 | `feat/web-auth-lobby` | Login (with "Play as guest"), register, lobby, leaderboard pages; extend the i18n module (saved language preference). |
| 9 | `feat/web-game` | Board rendering, controls, animation, bidding UI, chat; Playwright e2e with a registered player and a guest playing a round. |
| 10 | `chore/remove-legacy` | Delete `legacy/`, first moving `legacy/tools/board_oracle.py` to `tools/` with a copy of the board literal it reads, so the fixed-board test values stay regenerable; Dockerfile and README with run and test commands. |
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
| Guest play | Allowed without registration; games with a guest are unranked |
| Leaked data | Purged from git history; secret scanning added |
| Hosting | Undecided; investigate later. The Dockerfile in phase 10 stays host-agnostic. |
| Existing accounts and stats | Start fresh; nothing is imported from the legacy MySQL |

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
