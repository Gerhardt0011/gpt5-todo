# API Project Documentation

This project is a minimal yet scalable Axum-based REST API in Rust with a clean architecture and SQLite persistence. It provides a Todo resource with CRUD endpoints and tests (unit and acceptance) to validate behavior end-to-end.

## Goals
- Clear separation of concerns (HTTP vs. application logic vs. persistence)
- Maintainability and scalability for future resources
- Lightweight setup with SQLite
- Solid test coverage for core usage paths

## Tech Stack
- Axum 0.7 (HTTP server, routing)
- Tokio 1 (async runtime)
- SQLx 0.7 with SQLite (DB access)
- Serde (serialization)
- Tracing (logging)

## Project Layout
```
.
├── Cargo.toml
├── src
│   ├── main.rs                 # App bootstrap (wires routing + service + repo)
│   ├── lib.rs                  # Exposes modules for tests/integration
│   ├── domain                  # Enterprise/domain layer
│   │   ├── mod.rs
│   │   ├── todo.rs             # Todo entity, DTOs, enums
│   │   └── repository.rs       # TodoRepository trait
│   ├── application             # Application/service layer
│   │   ├── mod.rs
│   │   ├── todo_service.rs     # TodoService trait + impl
│   │   └── todo_service_tests.rs  # Unit tests for service (in-memory repo)
│   ├── infrastructure          # Adapters: databases, external services
│   │   ├── mod.rs
│   │   └── sqlite_repo.rs      # SQLx SQLite implementation of TodoRepository
│   └── http                    # Delivery/HTTP layer
│       ├── mod.rs              # Exposes http::routing and http::types
│       ├── types.rs            # API error/response helpers (extensible)
│       ├── routing             # Route composition & resource routers
│       │   ├── mod.rs          # app(router) adds health and merges routers
│       │   └── todos.rs        # Todos router and handlers
│       └── routes.rs           # Legacy placeholder (safe to delete)
│   └── bin
│       └── tui.rs              # Ratatui-based terminal UI to manage todos
├── tests
│   └── acceptance_todos.rs     # Acceptance/black-box tests against the router
└── docs
    └── README.md               # This document
```

Notes:
- `src/http/routes.rs` is a legacy placeholder kept empty to avoid module conflicts in this environment; you can delete it safely on disk. The new canonical location is `src/http/routing/*`.

## Architecture

Layers and roles:
- Domain (src/domain)
  - Entities and DTOs: `Todo`, `CreateTodo`, `UpdateTodo`, `TodoStatus`, `TodoId`
  - Repository ports: `TodoRepository` trait abstracts persistence
- Application (src/application)
  - `TodoService` trait and `TodoServiceImpl<R: TodoRepository>` implementation
  - Contains business/application logic; independent from HTTP and database
- Infrastructure (src/infrastructure)
  - `SqliteTodoRepository` uses SQLx to persist todos in SQLite
  - Responsible for schema creation at startup (`init`)
- HTTP (src/http)
  - Routing composition in `http::routing::app` (adds `/health` and merges routers)
  - Todos-specific router in `http::routing::todos::router`
  - Handlers map HTTP payloads to service calls and back to JSON

Benefits:
- Testable components in isolation (service unit tests, router acceptance tests)
- Easy to add new resources by adding new router modules and merging them in `app`
- Replaceable infrastructure (e.g., move to Postgres) with no changes to service/API

## API Surface

Base URL: `http://127.0.0.1:3000`

- GET `/health` -> 200 OK, body: `"ok"`

Todos
- POST `/todos`
  - Body: `{ "title": string, "description"?: string }`
  - 200 OK -> created todo
- GET `/todos`
  - 200 OK -> `{ "items": Todo[] }`
- GET `/todos/:id`
  - 200 OK -> todo | 404 if not found
- PUT `/todos/:id`
  - Body: `{ "title"?: string, "description"?: string, "status"?: "pending" | "done" }`
  - 200 OK -> updated todo | 404 if not found | 400 for invalid status
- DELETE `/todos/:id`
  - 204 No Content | 404 if not found

Todo JSON structure:
```
{
  "id": string (UUID),
  "title": string,
  "description": string | null,
  "status": "pending" | "done",
  "created_at": RFC3339 timestamp,
  "updated_at": RFC3339 timestamp
}
```

## TUI (Terminal UI)

We ship a fast, keyboard-driven TUI built with Ratatui to manage todos without starting the HTTP server.

Run the TUI:
```powershell
cargo run --bin tui
```

Features:
- Create todos (title and description)
- Edit title and description
- Toggle pending/done
- Delete todos
- Filter view: All, Pending, Done
- Details pane with title, status, and description

Keybindings:
- Up/Down: Move selection
- Enter: Toggle status pending <-> done
- n: Create mode
  - Type title/description
  - Tab: Switch field
  - Enter: Save, Esc: Cancel
- e: Edit selected
  - Prefills title/description
  - Tab: Switch field
  - Enter: Save, Esc: Cancel
- d: Delete selected
- f: Cycle filter (All → Pending → Done)
- q: Quit

## Persistence
- SQLite via SQLx. Default file path: `sqlite://todos.db` (override with `DATABASE_URL`).
- Schema is auto-created on startup by the repository’s `init` method.
- For tests, we use `sqlite::memory:`.

## Running & Testing

Run server:
```powershell
$env:DATABASE_URL = "sqlite://todos.db"; cargo run
```

Run TUI:
```powershell
cargo run --bin tui
```

Run tests:
```powershell
cargo test --all
```

Quick smoke (PowerShell):
```powershell
# Create
curl -s -Method POST http://127.0.0.1:3000/todos -ContentType 'application/json' -Body '{"title":"Test","description":"First"}'

# List
curl -s http://127.0.0.1:3000/todos

# Grab first id
$tid = (curl -s http://127.0.0.1:3000/todos | ConvertFrom-Json).items[0].id

# Get
curl -s http://127.0.0.1:3000/todos/$tid

# Update
curl -s -Method PUT http://127.0.0.1:3000/todos/$tid -ContentType 'application/json' -Body '{"status":"done"}'

# Delete
curl -s -Method DELETE http://127.0.0.1:3000/todos/$tid -i
```

## Testing Strategy
- Unit tests (service): `src/application/todo_service_tests.rs` uses an in-memory repo to test application logic.
- Acceptance tests (router): `tests/acceptance_todos.rs` drives requests against the Axum router using an in-memory SQLite database.

## Extending the API
1. Create `src/http/routing/<resource>.rs` with a `router(AppState { ... }) -> Router`.
2. In `main.rs`, merge it with `routing::app(existing.merge(<resource>::router(...)))`.
3. Add new domain models and repository methods as needed.
4. Add unit and acceptance tests mirroring what’s in todos.

## Observability
- Tracing via `tracing` and `tracing-subscriber` with `RUST_LOG` env (defaults to `info`).

## Environment Variables
- Managed via dotenv; `.env` is loaded automatically at startup (see `main.rs`).
- Copy `.env.example` to `.env` and adjust as needed. The `.env` file is git-ignored.
- `DATABASE_URL`: defaults to `sqlite://todos.db` if not set.
- `RUST_LOG`: e.g., `info,sqlx=warn`.

## Known Notes
- `src/http/routes.rs` is a placeholder that can be deleted; it’s empty to avoid module conflicts in this environment.
- If you need migrations, we can add `sqlx::migrate!()` with a `migrations/` directory.

## Troubleshooting
- Windows linking/tooling: if you run into `link.exe` not found, install “Build Tools for Visual Studio” with the “Desktop development with C++” workload.
- Database file: if using a file URL like `sqlite://todos.db`, the app will create the file if needed. For ephemeral use, `sqlite::memory:` works too.
