# issuedesk

A simple, Jira-like ticketing tool: projects as scope, issues/bugs tracked per
project. Deliberately small — Jira's core (projects, issues, board, comments,
labels, attachments, activity) without the platform sprawl.

## Stack

- **Backend:** Rust + Axum 0.8, SQLx 0.8 (Postgres, compile-time-checked queries),
  JWT auth (HS256), argon2 password hashing. Single binary, also serves the SPA.
- **Frontend:** SvelteKit 5 (runes), `adapter-static`, hash router, Tailwind v4.
- **DB:** PostgreSQL 16.
- **Deploy:** one app image (multi-stage: node build → rust build → slim runtime)
  + a postgres service, via docker-compose.

## Layout

```
solutions/issuedesk-api/   # Rust backend (Axum + SQLx)
  src/                     # config, state, error, router, auth/, handlers/, db/, models/, dto/
  migrations/              # SQLx migrations 0001..0005 (run at startup)
  .sqlx/                   # committed offline query cache (for SQLX_OFFLINE builds)
solutions/issuedesk-web/   # SvelteKit 5 frontend (built into /app/static at image build)
Dockerfile                 # 3-stage build
docker-compose.yml         # postgres + issuedesk
```

## Conventions (inherited from rpos/fms)

- **Auth:** stateless JWT Bearer in `Authorization` header. `POST /auth/signIn`
  `{userName, password}` → `{token, user}`. All `/api/**` routes are guarded.
  Browser-lock: requests must carry `Sec-Fetch-Site` ∈ {same-origin, same-site,
  none}. **Testing with curl/Postman must add `-H 'Sec-Fetch-Site: same-origin'`.**
- **Enums** are int-backed (`smallint` in PG, `#[repr(i16)]` in Rust).
- **IDs** are UUIDs; the public issue identifier is `KEY-number` (e.g. `WAT-1`),
  with the per-project number assigned atomically via `UPDATE projects SET
  issue_seq = issue_seq + 1 ... RETURNING`.
- **Config** is env-driven; `DATABASE_URL` is composed from `APP_SERVER/PORT/
  DATABASE/USER/PASSWORD`.
- **Migrations** run on boot (`sqlx::migrate!()`); an admin user is seeded when the
  users table is empty and `SEED_ADMIN=true`.
- **Commit messages:** `claude - <short topic>`.

## Develop

```bash
# 1. Postgres (compose just the db, or use your own)
docker compose up -d postgres

# 2. Backend
cd solutions/issuedesk-api
cp ../../.env.example .env        # adjust if needed
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/issuedeskdevl
cargo sqlx migrate run            # or rely on startup migration
cargo run                         # serves API + (if built) SPA on :8080

# After changing any SQL query, refresh the offline cache and commit it:
cargo sqlx prepare

# 3. Frontend (dev)
cd solutions/issuedesk-web
npm install
npm run dev                       # Vite dev server, proxies /api + /auth to :8080
```

## Run everything (prod-like)

```bash
cp .env.example .env
docker compose up --build
# open http://localhost:8080  — log in with the seeded admin (default admin/admin123)
```

## Build the whole image without a live DB

The Rust build uses `SQLX_OFFLINE=true` and the committed `.sqlx/` cache, so
`docker compose up --build` needs no database at build time. If you change queries
and forget `cargo sqlx prepare`, the build fails with a stale-cache error.
