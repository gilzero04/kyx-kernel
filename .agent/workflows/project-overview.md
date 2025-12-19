# Kyx Kernel - Project Overview
// turbo-all

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Backend** | Rust, ntex (async web framework) |
| **Database** | PostgreSQL 16 |
| **Cache** | Redis |
| **Auth** | JWT (access + refresh tokens) |
| **Container** | Docker, docker-compose |
| **Frontend** | Svelte 5, SvelteKit, Bun, TailwindCSS |

## Architecture

```
kyx-kernel/                 # Backend (Rust)
├── src/
│   ├── core/               # Shared infrastructure
│   │   ├── infrastructure/ # DB, Redis, CORS, Audit, etc.
│   │   ├── domain/         # Core entities, traits
│   │   └── utils/          # JWT, validation helpers
│   └── modules/            # Feature modules
│       ├── auth/           # Authentication, Users
│       └── system/         # Config, API Keys, CORS
├── infra/docker/           # Dockerfiles
└── .agent/workflows/       # Development documentation

kyx-platform/               # Frontend (SvelteKit)
└── src/routes/             # Pages and layouts
```

## Key Design Decisions

1. **Modular Architecture** - Each feature is a self-contained module
2. **Plugin-Ready** - Core is minimal, extensions via plugins
3. **Multi-Tenant** - Hierarchical tenant structure (parent_id)
4. **Soft Delete** - All entities use `deleted_at` for recoverability
5. **Scoped RBAC** - Same role, different scope per tenant

## Environment Variables

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `REDIS_* ` | Redis connection |
| `ENGINE_SECRET_KEY` | Secret for setup/internal API |
| `JWT_SECRET` | JWT signing key |

## Running the Project

```bash
# Start databases
docker-compose up -d

# Run Kernel
cargo run --bin kyx-kernel

# Run Platform (in kyx-platform/)
bun run dev
```
