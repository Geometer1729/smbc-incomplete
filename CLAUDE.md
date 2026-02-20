# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A Rust game engine for the "Incomplete" game from SMBC comics — a Tic-Tac-Toe variant where players have hidden preference orderings over outcomes (win/draw/loss). Web frontend communicates with a Rust backend over WebSocket.

## Build & Run

```bash
cargo build           # Build (debug)
cargo build --release # Build (release)
cargo run             # Run server (starts WebSocket on 127.0.0.1:8080)
cargo bench           # Run benchmarks (criterion)
cargo fmt             # Format code
cargo clippy          # Lint
```

The dev environment is managed via Nix flake + direnv (`use flake`). Entering the directory automatically loads the Rust toolchain with rust-analyzer, rustfmt, and clippy.

Open `web/game.html` in a browser after `cargo run` to play via the web UI.

## Architecture

### Game Loop (`api.rs`)

`host_game()` is the core recursive game loop. It takes two `&mut impl API` players (one for X, one for O) and alternates turns. The `API` trait defines three methods:
- `rend(Pos)` — render the current position to the player
- `ask(Pos)` — request a move from the player (returns new `Pos`)
- `close(Outcome)` — notify player the game ended

Three `API` implementations exist:
- `Cnsl` — console player (stdin/stdout) in `api.rs`
- `SimpleAi` — AI player in `ai.rs`, selects moves based on a ranked `Objective` (preference ordering over outcomes)
- `WebPlayer` — browser player via WebSocket in `web.rs`

### Evaluation Table (`table.rs`, `eval.rs`, `cannon.rs`)

On startup, `gen_table()` pre-computes evaluations for all reachable board positions using rayon for parallelism, stored in a `DashMap<Pos, Eval>`. The `Eval` type is `[[bool; 3]; 2]` — tracks which outcomes each player can force or prevent.

`cannon.rs` reduces positions to canonical form via board symmetries (rotation/reflection), shrinking the table ~8x.

### WebSocket Protocol (`web.rs`, `web/game.js`)

- Server listens on `127.0.0.1:8080`
- Server→client: text messages in format `"xXOXOXXOX"` (turn char + 9 board squares)
- Client→server: binary single-byte move index (0–8) for board position
- Ping/pong handling via `ReqPong`/`InMsg` message types
- Uses tokio mpsc channels to bridge async WebSocket I/O with the game loop

### Key Types (`types.rs`)

- `Pos { turn: Player, board: Board }` — game state
- `Board = [Square; 9]` — the 3x3 board as a flat array
- `Eval = [[bool; 3]; 2]` — position evaluation matrix
- `Objective = [Outcome; 3]` — ranked preference ordering for AI
- `Pref { Lost, Drawable, Winable }` — evaluation of a position from a player's perspective

### Frontend (`web/`)

Vanilla JS with SVG rendering. `Board` class handles the grid display; `Pos` class parses server messages. Connects to `ws://localhost:8080/`.
