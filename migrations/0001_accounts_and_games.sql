-- Accounts and finished games. Timestamps are milliseconds; ids are the
-- 128-bit PlayerId as 32 lowercase hex digits.

-- Players sign in with Google; only Google's stable account id is kept.
CREATE TABLE accounts (
    id             TEXT    PRIMARY KEY NOT NULL,
    google_subject TEXT    NOT NULL UNIQUE,
    username       TEXT    NOT NULL,
    -- Lowercased copy enforces uniqueness ignoring case.
    username_key   TEXT    NOT NULL UNIQUE,
    locale         TEXT    NOT NULL CHECK (locale IN ('en', 'hr')),
    created_at     INTEGER NOT NULL
);

CREATE TABLE games (
    id          INTEGER PRIMARY KEY,
    finished_at INTEGER NOT NULL,
    -- Fixed when stored: two or more players, all with accounts.
    ranked      INTEGER NOT NULL CHECK (ranked IN (0, 1))
);

CREATE TABLE game_players (
    game_id    INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    account_id TEXT    REFERENCES accounts (id),
    guest_name TEXT,
    tokens     INTEGER NOT NULL CHECK (tokens >= 0),
    won        INTEGER NOT NULL CHECK (won IN (0, 1)),
    -- Exactly one of account or guest; guests are never linked to accounts.
    CHECK ((account_id IS NULL) <> (guest_name IS NULL))
);

CREATE INDEX game_players_by_account ON game_players (account_id);
CREATE INDEX game_players_by_game ON game_players (game_id);
