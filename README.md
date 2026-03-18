# AUTH

A stateless authentication service built in Rust, featuring JWT-based access and refresh token flows backed by PostgreSQL.

---

## Table of Contents

- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Getting Started](#getting-started)
- [Configuration](#configuration)
- [RSA Key Generation](#rsa-key-generation)
- [Database Migrations](#database-migrations)
- [Running the Application](#running-the-application)
- [Environment Variables](#environment-variables)

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| Web Framework | Axum |
| Database | PostgreSQL |
| ORM / Migrations | SQLx |
| Authentication | JWT (RS256) |
| Config Templating | Tera |
| Logging | tracing / tracing-subscriber |

---

## Project Structure

```
.
├── config/                  # Environment-specific YAML configs
│   ├── development.yaml
│   ├── production.yaml
│   └── testing.yaml
├── migrations/              # SQLx migration files (up/down)
├── secrets/keys/            # RSA key pairs (not committed to VCS)
├── src/
│   ├── bin/main.rs          # Entrypoint
│   ├── app.rs               # CLI parsing and server startup
│   ├── context.rs           # Shared application state (AppContext)
│   ├── config/              # Config loading and validation
│   ├── controllers/         # Route handlers
│   ├── middlewares/         # Request tracing, JSON extraction
│   ├── repository/          # Database access layer
│   ├── validator/           # Input validation
│   └── views/               # Response serialization
├── compose.yaml             # Docker Compose (PostgreSQL)
└── Cargo.toml
```

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [PostgreSQL](https://www.postgresql.org/) running on port `5432`
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli) for managing migrations

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Alternatively, spin up PostgreSQL via Docker Compose:

```bash
docker compose up -d
```

### Installation

```bash
# Clone the repository
git clone https://github.com/<your-username>/auth.git
cd auth

# Build the project
cargo build
```

---

## Configuration

The application reads configuration from `config/<environment>.yaml` at startup. The environment is selected via the `--env` CLI flag (default: `development`).

Config files are rendered as [Tera](https://keats.github.io/tera/) templates, allowing environment variables to be interpolated at runtime using `get_env()`.

**Example — `config/development.yaml`:**

```yaml
server:
  protocol: http
  host: {{ get_env(name = "SERVER_HOST", default = "127.0.0.1") }}
  port: {{ get_env(name = "SERVER_PORT", default = "7150") }}

database:
  uri: {{ get_env(name = "DATABASE_URL", default = "postgresql://username:password@localhost:5432/database") }}
  max_connections: {{ get_env(name = "DATABASE_MAX_CONNECTIONS", default = "10") }}
  min_connections: {{ get_env(name = "DATABASE_MIN_CONNECTIONS", default = "0") }}
  connection_timeout: {{ get_env(name = "DATABASE_CONNECTION_TIMEOUT", default = "5") }}
  idle_timeout: {{ get_env(name = "DATABASE_IDLE_TIMEOUT", default = "5") }}
  auto_migrate: {{ get_env(name = "DATABASE_AUTO_MIGRATE", default = "true") }}
  dangerously_truncate: {{ get_env(name = "DATABASE_DANGEROUSLY_TRUNCATE", default = "false") }}
  dangerously_recreate: {{ get_env(name = "DATABASE_DANGEROUSLY_RECREATE", default = "false") }}

logger:
  level: debug
  format: pretty
  crates:
    - auth
    - axum
    - sqlx
    - tower
    - tower_http

auth:
  access:
    private_key: secrets/keys/dev/access_key.pem
    public_key: secrets/keys/dev/access_key_pub.pem
    maxage: 900       # 15 minutes
  refresh:
    private_key: secrets/keys/dev/refresh_key.pem
    public_key: secrets/keys/dev/refresh_key_pub.pem
    maxage: 604800    # 7 days
```

### Database Flags

The following flags in the `database` section control migration behaviour. Use with caution outside of development:

| Flag | Description |
|---|---|
| `auto_migrate` | Automatically run pending migrations on startup |
| `dangerously_truncate` | Truncate all tables (planned — not yet implemented) |
| `dangerously_recreate` | Roll back all migrations and re-run them from scratch |

> ⚠️ **Warning:** `dangerously_recreate: true` will **drop and recreate all tables**. Never enable this in production.

---

## RSA Key Generation

AUTH uses **RS256** (RSA + SHA-256) for signing both access and refresh JWTs. You must generate two separate key pairs — one for each token type — and place them in the `secrets/keys/dev/` directory.

```bash
mkdir -p secrets/keys/dev

# Access token key pair
openssl genrsa -out secrets/keys/dev/access_key.pem 2048
openssl rsa -in secrets/keys/dev/access_key.pem \
            -pubout -out secrets/keys/dev/access_key_pub.pem

# Refresh token key pair
openssl genrsa -out secrets/keys/dev/refresh_key.pem 2048
openssl rsa -in secrets/keys/dev/refresh_key.pem \
            -pubout -out secrets/keys/dev/refresh_key_pub.pem
```

The expected directory layout after generation:

```
secrets/keys/dev/
├── access_key.pem
├── access_key_pub.pem
├── refresh_key.pem
└── refresh_key_pub.pem
```

> **Important:** Never commit private keys to version control. Add `secrets/` to your `.gitignore`.

---

## Database Migrations

Migrations live in the `migrations/` directory and are managed by SQLx. Each migration has an `up` and a `down` file.

Run migrations manually:

```bash
sqlx migrate run
```

Roll back the latest migration:

```bash
sqlx migrate revert
```

Alternatively, set `auto_migrate: true` in your config to have migrations run automatically on startup.

---

## Running the Application

The application reads a `.env` file from the project root if one is present (via `dotenvy`). Create one to override config defaults:

```env
DATABASE_URL=postgresql://username:password@localhost:5432/auth
SERVER_HOST=127.0.0.1
SERVER_PORT=7150
```

Then start the server:

```bash
# Development (default)
cargo run

# Specify an environment explicitly
cargo run -- --env production
```

Available `--env` values: `development` (or `dev`), `production` (or `prod`), `testing` (or `test`).

On a successful start you should see:

```
INFO auth: Server running at http://127.0.0.1:7150
```

---

## Environment Variables

All variables below have defaults defined in the config templates and are optional unless marked otherwise.

| Variable | Default | Description |
|---|---|---|
| `SERVER_HOST` | `127.0.0.1` | Host the HTTP server binds to |
| `SERVER_PORT` | `7150` | Port the HTTP server listens on |
| `DATABASE_URL` | `postgresql://username:password@localhost:5432/database` | PostgreSQL connection string |
| `DATABASE_MAX_CONNECTIONS` | `10` | Maximum connections in the pool |
| `DATABASE_MIN_CONNECTIONS` | `0` | Minimum idle connections in the pool |
| `DATABASE_CONNECTION_TIMEOUT` | `5` | Seconds to wait for a connection |
| `DATABASE_IDLE_TIMEOUT` | `5` | Seconds before an idle connection is closed |
| `DATABASE_AUTO_MIGRATE` | `true` | Run migrations on startup |
| `DATABASE_DANGEROUSLY_TRUNCATE` | `false` | Truncate all tables on startup |
| `DATABASE_DANGEROUSLY_RECREATE` | `false` | Drop and recreate schema on startup |
