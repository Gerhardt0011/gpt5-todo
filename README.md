# Todo API + TUI (Axum + SQLx + Ratatui)

A small, layered Todo application:
- HTTP API built with Axum (SQLite via SQLx)
- Terminal User Interface (TUI) built with Ratatui
- Clean separation between HTTP and application logic, with tests

## Quickstart (Windows PowerShell)

Prerequisites:
- Rust toolchain (stable)
- On Windows, if you hit linker issues (link.exe), install "Build Tools for Visual Studio" with the "Desktop development with C++" workload.

1) Configure environment
- Copy `.env.example` to `.env` and customize, or set `DATABASE_URL` directly. Example values:
  - `sqlite://todos.db` (file-backed, persistent)
  - `sqlite::memory:` (in-memory, ephemeral)

2) Build
```
cargo build
```

3) Run the HTTP API
- With `.env` present: `cargo run`
- Or via PowerShell in one line:
```
$env:DATABASE_URL = "sqlite://todos.db"; cargo run
```

4) Run the TUI
```
cargo run --bin tui
```

5) Run tests
```
cargo test --all
```

## API Overview

Base URL: `http://localhost:3000`

- Health: `GET /health` → `{ "status": "ok" }`
- Create Todo: `POST /todos` with body:
  ```json
  { "title": "Buy milk", "description": "Full-cream", "status": "Pending" }
  ```
- List Todos: `GET /todos`
- Get by ID: `GET /todos/:id`
- Update: `PUT /todos/:id` with body:
  ```json
  { "title": "Buy milk and eggs", "description": "Free-range", "status": "Done" }
  ```
- Delete: `DELETE /todos/:id`

Todo JSON:
```json
{
  "id": "<uuid>",
  "title": "...",
  "description": "...", // optional
  "status": "Pending" | "Done",
  "created_at": "<rfc3339>",
  "updated_at": "<rfc3339>"
}
```

## TUI Overview

Manage todos without the HTTP server. Start it with:
```
cargo run --bin tui
```

Highlights:
- Create/edit title and description
- Toggle Pending/Done with Enter
- Delete
- Filter between All / Pending / Done
- Details pane shows title, status, and description

Keys:
- Up/Down: navigate
- Enter: toggle status
- n: create (Tab to switch fields, Enter to save, Esc to cancel)
- e: edit (Tab to switch fields, Enter to save, Esc to cancel)
- d: delete
- f: cycle filter
- q: quit

More details are available in `docs/README.md`.

## Project Structure

```
.
├── Cargo.toml
├── README.md
├── docs/
│   └── README.md        # Architecture, API, TUI, troubleshooting
├── src/
│   ├── bin/
│   │   └── tui.rs       # Ratatui TUI entrypoint
│   ├── http/            # HTTP layer (routing, handlers)
│   ├── application/     # Services (business logic)
│   ├── domain/          # Entities & repository traits
│   ├── infrastructure/  # SQLx SQLite repository
│   └── main.rs          # API server bootstrap
└── tests/
    └── acceptance_todos.rs
```

## Troubleshooting
- Windows linking: install VS Build Tools if you see `link.exe` errors.
- SQLite file creation: the app will prepare the SQLite file/dirs automatically for `sqlite://...` URLs. In-memory is `sqlite::memory:`.
- Logging: set `RUST_LOG="info,sqlx=warn"` for helpful runtime logs.
