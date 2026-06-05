# issuedesk

A simple, self-hosted issue tracker — projects as scope, issues/bugs tracked per
project. Jira's core (board, list, comments, labels, attachments, activity)
without the platform sprawl.

- **Backend:** Rust + [Axum](https://github.com/tokio-rs/axum) + [SQLx](https://github.com/launchbadge/sqlx) (PostgreSQL), JWT auth, argon2 password hashing
- **Frontend:** [SvelteKit 5](https://svelte.dev) (runes) + Tailwind CSS v4, single-page app
- **Deploy:** one Docker image (the Rust binary also serves the built SPA) + a Postgres service

The whole app ships as a single ~100 MB container.

## Features

- **Projects** as scopes, each with a key (e.g. `WAT`); issues get per-project numbers (`WAT-1`, `WAT-2`, …) assigned race-free
- **Issues** with type (bug/task/story/epic), status, priority, assignee, reporter, markdown description
- **Kanban board** with drag-between-columns, plus a filterable/sortable **list** view
- **Comments**, **labels** (colored, per-project), **file attachments**, and a per-issue **activity log**
- **Auth & roles:** JWT bearer tokens, global admin/member roles, per-project membership; admins manage users and all projects
- Browser-lock via `Sec-Fetch-Site` (matches the convention used across sibling projects)

## Quick start (Docker)

```bash
git clone git@github.com:tofilagman/issuedesk.git
cd issuedesk
cp .env.example .env          # set a real JWT_SECRET
docker compose up --build     # → http://localhost:8080
```

Log in with the seeded admin (default `admin` / `admin123`) and **change the
password immediately**. The admin is only seeded on first boot, while the users
table is empty (gated by `SEED_ADMIN`).

There's also a [`docker-compose.dokploy.yml`](docker-compose.dokploy.yml) for
deploying the published image (`tofilagman/issuedesk`) via
[Dokploy](https://dokploy.com).

## Development

Requires Rust (stable), Node 22, and a PostgreSQL instance.

```bash
# 1. Start Postgres (or use your own) and create the dev DB
docker compose up -d postgres
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/issuedeskdevl

# 2. Backend (serves API + the built SPA on :8080)
cd solutions/issuedesk-api
sqlx migrate run          # or rely on the startup migration
cargo run

# 3. Frontend dev server (proxies /api + /auth to :8080)
cd solutions/issuedesk-web
npm install
npm run dev
```

### SQLx offline cache

Queries are checked at compile time against the database. The generated `.sqlx/`
cache is committed so the Docker build works with `SQLX_OFFLINE=true` and no live
DB. **After changing any SQL query, regenerate and commit it:**

```bash
cd solutions/issuedesk-api
cargo sqlx prepare
```

## Configuration

All configuration is via environment variables. `DATABASE_URL` is composed from
the `APP_*` parts if not set explicitly.

| Variable | Default | Description |
| --- | --- | --- |
| `APP_SERVER` / `APP_PORT` | `localhost` / `5432` | Postgres host / port |
| `APP_DATABASE` | `issuedeskdevl` | Database name |
| `APP_USER` / `APP_PASSWORD` | `postgres` / `postgres` | Postgres credentials |
| `DATABASE_URL` | _(composed)_ | Overrides the `APP_*` parts if set |
| `PORT` | `8080` | HTTP listen port |
| `JWT_SECRET` | _(required)_ | Signing secret — set a long random value |
| `JWT_TTL_HOURS` | `24` | Token lifetime |
| `UPLOAD_DIR` | `./data/uploads` | Where attachments are stored on disk |
| `MAX_UPLOAD_BYTES` | `26214400` | Max attachment size (25 MB) |
| `STATIC_DIR` | `./static` | Directory of the built SPA to serve |
| `SEED_ADMIN` | `false` | Seed an admin when the users table is empty |
| `SEED_ADMIN_USERNAME` / `SEED_ADMIN_EMAIL` / `SEED_ADMIN_PASSWORD` | `admin` / … / `admin123` | Seed admin details |
| `RUST_LOG` | `info` | Log filter |

> **Note:** mount a volume at `UPLOAD_DIR` in production so attachments survive
> redeploys.

## Project layout

```
solutions/issuedesk-api/   # Rust backend (Axum + SQLx)
  src/                     # config, state, error, router, auth/, handlers/, db/, models/, dto/
  migrations/              # SQLx migrations (run at startup)
  .sqlx/                   # committed offline query cache
solutions/issuedesk-web/   # SvelteKit 5 frontend (built into the image)
Dockerfile                 # 3-stage build: node → rust → slim runtime
docker-compose.yml         # local: postgres + app
docker-compose.dokploy.yml # deploy the published image
```

## API overview

`POST /auth/signIn` is open; everything under `/api/**` requires a bearer token
and a `Sec-Fetch-Site` header. Testing with curl/Postman:

```bash
curl -H 'Sec-Fetch-Site: same-origin' -H 'Content-Type: application/json' \
  -d '{"userName":"admin","password":"admin123"}' http://localhost:8080/auth/signIn
```

Resources: `users`, `projects`, project `members`, `issues` (list/filter, create,
patch — the patch drives the board), `comments`, `labels`, `attachments`,
`activity`. See [`solutions/issuedesk-api/src/router.rs`](solutions/issuedesk-api/src/router.rs)
for the full route table.
