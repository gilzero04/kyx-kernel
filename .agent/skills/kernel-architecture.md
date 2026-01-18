---
description: Kyx Kernel Architecture - Complete project structure and flow documentation
---

# Kyx Kernel Architecture Skill

> สรุปโครงสร้างโปรเจค และกระบวนการทำงานทั้งหมดของ kyx-kernel

## When to Use

- ต้องการเข้าใจโครงสร้าง project
- Debug ปัญหาใน specific module
- เพิ่ม feature ใหม่
- Trace request flow

---

## High-Level Architecture

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                          Kyx Kernel Architecture                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                         main.rs (Entry Point)                            │ │
│  │  • Kernel::init() → Bootstrap infrastructure                             │ │
│  │  • web::server() → Start HTTP server                                     │ │
│  │  • Module configuration via try_configure()                              │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                        │
│  ┌───────────────────────────────────▼───────────────────────────────────┐   │
│  │                              CORE                                      │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐  │   │
│  │  │  bootstrap  │ │infrastructure│ │    utils    │ │    domain       │  │   │
│  │  │  ├─ Kernel  │ │├─ database  │ │├─ jwt       │ │├─ error.rs      │  │   │
│  │  │  ├─ Registry│ │├─ redis     │ │├─ password  │ │└─ AppError      │  │   │
│  │  │  └─ infra   │ │├─ audit     │ │├─ avatar    │ │                 │  │   │
│  │  │             │ │├─ cors      │ │└─ seeding   │ │                 │  │   │
│  │  │             │ │├─ rate_limit│ │             │ │                 │  │   │
│  │  │             │ │└─ config    │ │             │ │                 │  │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────┘  │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                      │                                        │
│  ┌───────────────────────────────────▼───────────────────────────────────┐   │
│  │                            MODULES                                     │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐  │   │
│  │  │    auth     │ │   system    │ │    media    │ │     signal      │  │   │
│  │  │ ├─ login    │ │├─ tenant    │ │├─ upload    │ │├─ token exchange│  │   │
│  │  │ ├─ register │ │├─ user      │ │├─ folder    │ │└─ WebSocket prep│  │   │
│  │  │ ├─ session  │ │├─ role      │ │└─ assets    │ │                 │  │   │
│  │  │ ├─ setup    │ │├─ theme     │ │             │ │                 │  │   │
│  │  │ └─ logout   │ │├─ branding  │ │             │ │                 │  │   │
│  │  │             │ │├─ plugin    │ │             │ │                 │  │   │
│  │  │             │ │├─ cms       │ │             │ │                 │  │   │
│  │  │             │ │├─ i18n      │ │             │ │                 │  │   │
│  │  │             │ │└─ share     │ │             │ │                 │  │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────┘  │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Project Structure

```
kyx-kernel/
├── src/
│   ├── main.rs                 # Entry point, HTTP server
│   ├── core/                   # Shared infrastructure
│   │   ├── mod.rs              # Core exports
│   │   ├── bootstrap/          # Kernel initialization
│   │   │   ├── mod.rs          # Kernel struct
│   │   │   ├── infrastructure.rs # DB/Redis init
│   │   │   └── registry.rs     # Service wiring
│   │   ├── infrastructure/     # Cross-cutting concerns
│   │   │   ├── database.rs     # PostgreSQL pool
│   │   │   ├── redis.rs        # Redis connection
│   │   │   ├── audit.rs        # Audit logging
│   │   │   ├── cors.rs         # CORS management
│   │   │   ├── rate_limit.rs   # Rate limiting
│   │   │   ├── config_service.rs # Dynamic config
│   │   │   └── permission_middleware.rs # RBAC
│   │   ├── utils/              # Utilities
│   │   │   ├── jwt.rs          # JWT generation/validation
│   │   │   ├── password.rs     # Password hashing
│   │   │   └── avatar.rs       # Avatar generation
│   │   └── domain/             # Core domain types
│   │       └── error.rs        # AppError
│   ├── modules/                # Business Modules
│   │   ├── mod.rs
│   │   ├── auth/               # Authentication
│   │   ├── system/             # System management
│   │   ├── media/              # Media library
│   │   └── signal/             # Real-time bridge
│   └── interface/              # Shared HTTP types
│       └── http/
│           └── openapi.rs      # OpenAPI spec
├── migrations/                 # SQL migrations (0001-0011)
├── assets/                     # Static files
│   └── themes/presets/         # Theme bundles
├── .agent/                     # AI agent config
│   ├── skills/                 # Skill documents
│   └── workflows/              # Workflow definitions
└── Cargo.toml
```

---

## Module Layer Architecture

```
Each Module follows Clean Architecture:

┌───────────────────────────────────────────────────────────────┐
│                         MODULE                                 │
├───────────────────────────────────────────────────────────────┤
│                                                                │
│  interface/http/           (Outer Layer - HTTP)               │
│  ├── routers/              Route definitions                  │
│  ├── handlers/             Request handlers                   │
│  └── dto/                  Data transfer objects              │
│                                                                │
│  application/services/     (Application Layer)                │
│  └── {service}/            Business logic                     │
│                                                                │
│  infrastructure/           (Infrastructure Layer)             │
│  └── repositories/         Database access                    │
│                                                                │
│  domain/                   (Domain Layer - Core)              │
│  └── {entity}.rs           Entities, Traits                   │
│                                                                │
└───────────────────────────────────────────────────────────────┘
```

---

## Authentication Flow

### Login Process

```
1. POST /api/v1/auth/login
   Body: { username, password }
        │
2. Handler: login() (handlers/auth/mod.rs)
        │
3. Service: AuthService.login() (services/auth/mod.rs)
   ├── Find user by email (auth_users)
   ├── Verify password (bcrypt)
   ├── Get membership + role (auth_memberships + sys_roles)
   ├── Fetch permissions (sys_role_permissions + sys_permissions)
   ├── Generate JWT tokens (access + refresh)
   ├── Store session in Redis (auth:session:{user_id}:{sid})
   └── Audit log (LOGIN_SUCCESS)
        │
4. Return: AuthResponse
   {
     access_token: "...",
     refresh_token: "...",
     user: { id, email, role, permissions, tenant_type }
   }
```

### Session Management

```
Redis Key Pattern: auth:session:{user_id}:{session_id}

Session Info:
{
  sid: "uuid",
  ip: "192.168.1.1",
  user_agent: "Chrome/...",
  created_at: timestamp,
  expires_at: timestamp
}

Operations:
- Login: SET session with TTL
- Refresh: GET session, extend TTL
- Logout: DEL session
- List Sessions: KEYS auth:session:{user_id}:*
```

### JWT Token Structure

```rust
Claims {
    sub: "user_id",          // Subject (user UUID)
    role: "superadmin",      // Role slug
    tenant_id: UUID,         // Current tenant
    permissions: ["..."],    // Permission slugs
    is_system_owner: bool,   // Platform owner flag
    sid: "session_id",       // Session ID for revocation
    token_type: Access|Refresh,
    exp: timestamp,
    iat: timestamp,
}
```

---

## System Setup Flow

```
1. GET /api/v1/auth/setup/status
   └── Check if auth_users has any records
        │
2. POST /api/v1/auth/setup  (First time only!)
   Body: SetupRequest {
     email, password, full_name,
     org_name, org_slug, app_name,
     platform_type, ...
   }
        │
3. AuthService.initialize_system() (Transaction)
   ├── Validate password policy
   ├── Create auth_tenants (owner, parent_id = id)
   ├── Create sys_brandings (console + workspace)
   ├── Create sys_roles (superadmin, admin, operator, viewer)
   ├── Create auth_users (first admin)
   ├── Create auth_memberships (admin → superadmin)
   ├── Seed default homepage in sys_pages
   ├── Handover orphaned records (see below)
   ├── Set default themes in branding
   └── Generate JWT tokens
        │
4. Return: AuthResponse (auto-login)
```

### Orphaned Records Handover

> **Critical:** หลัง setup ต้อง assign tenant_id ให้ records ที่มี NULL

| Table                   | Location            | หมายเหตุ                       |
| ----------------------- | ------------------- | ------------------------------ |
| `sys_pages`             | auth/mod.rs:684     | Default homepage               |
| `sys_themes`            | auth/mod.rs:691     | System themes (Kyx Light/Dark) |
| `sys_roles`             | auth/mod.rs:697-707 | Skip duplicates                |
| `sys_api_keys`          | auth/mod.rs:710     | Pre-seeded keys                |
| `sys_configs`           | auth/mod.rs:717     | Platform configs               |
| `sys_plugins`           | auth/mod.rs:722     | Pre-installed plugins          |
| `sys_i18n_translations` | auth/mod.rs:728     | Default translations           |

**Not Handed Over (by design):**

- `sys_brandings` - Created during setup, not pre-seeded

---

## Request Flow

```
HTTP Request → Ntex Router → Middleware → Handler → Service → Repository → PostgreSQL
                    │
                    ├── CORS Middleware (DynamicCors)
                    ├── Rate Limit (DynamicRateLimit)
                    └── Permission Middleware (RequirePermission)
                              │
                              ├── Extract JWT from Authorization header
                              ├── Verify token signature + expiry
                              ├── Check required permission
                              └── Inject Claims into request extensions
```

---

## API Structure

```
/api/v1/
├── auth/                   # Auth module
│   ├── login               # POST - Login
│   ├── logout              # POST - Logout
│   ├── refresh             # POST - Refresh token
│   ├── setup/              # Setup endpoints
│   │   ├── status          # GET - Check if setup done
│   │   └── (root)          # POST - Initialize system
│   ├── sessions/           # Session management
│   │   ├── (list)          # GET - List own sessions
│   │   └── {sid}           # DELETE - Revoke session
│   └── preferences/        # User preferences
│
├── admin/                  # System module (protected)
│   ├── users/              # User CRUD
│   ├── tenants/            # Tenant CRUD
│   ├── roles/              # Role management
│   ├── permissions/        # Permission management
│   ├── themes/             # Theme management
│   ├── brandings/          # Branding management
│   ├── plugins/            # Plugin management
│   ├── cms/                # CMS pages
│   ├── i18n/               # Translations
│   ├── shares/             # Resource sharing
│   └── system/             # System info
│
├── public/                 # Public endpoints (no auth)
│   ├── themes              # GET - List available themes
│   ├── home                # GET - CMS home page
│   └── branding            # GET - Tenant branding
│
├── media/                  # Media module
│   ├── upload              # POST - Upload file
│   ├── folders/            # Folder management
│   └── assets/             # Asset management
│
└── signal/                 # Signal bridge
    └── exchange            # POST - Token exchange for WebSocket
```

---

## Key Infrastructure Services

### ConfigService

```rust
// Dynamic configuration from sys_configs table
config_service.get_int("access_token_expire_minutes", 30).await
config_service.set("key", value).await

// Scopes: platform, tenant
```

### AuditService

```rust
// Audit logging to audit_logs table
audit_service.log(&user_id, "ACTION", Some(&target), "SUCCESS|FAILURE", metadata).await
```

### CorsManager

```rust
// Dynamic CORS from sys_cors_origins table
cors_manager.get_allowed_origins() // Vec<String>
```

### Permission Middleware

```rust
// Wrap routes with permission check
RequirePermission::new("permission:slug", jwt, audit)
    .with_redis(redis) // Session validation

// Empty string = just require valid JWT
RequirePermission::new("", jwt, audit)
```

---

## Database Tables by Module

### Auth Module

- `auth_users` - User accounts
- `auth_memberships` - User-Tenant-Role mapping
- `user_preferences` - User settings

### System Module

- `auth_tenants` - Organizations
- `sys_tenant_types` - Tenant type definitions
- `sys_roles` - RBAC roles
- `sys_permissions` - RBAC permissions
- `sys_role_permissions` - Role-Permission mapping
- `sys_themes` - Theme registry
- `sys_brandings` - Branding configurations
- `sys_plugins` - Plugin registry
- `sys_pages` - CMS pages
- `sys_i18n_locales` - Locales
- `sys_i18n_translations` - Translations
- `sys_resource_shares` - Sharing system
- `sys_configs` - Dynamic configuration
- `sys_cors_origins` - CORS whitelist
- `sys_api_keys` - API keys
- `audit_logs` - Audit trail

### Media Module

- `media_folders` - Folder structure
- `media_assets` - Media files

---

## Environment Variables

```bash
# Server
PORT=8080
ENVIRONMENT=local|staging|production

# Database
DATABASE_URL=postgresql://user:pass@host:5432/db

# Redis
REDIS_URL=redis://localhost:6379

# Security
JWT_SECRET=your-secret-key
ENGINE_SECRET=handshake-secret

# API Docs (staging/prod only)
DOCS_USER=admin
DOCS_PASSWORD=secret
```

---

## Related Skills

- `entity-data-reference.md` - Database schema reference
- `theme-api-flow.md` - Theme system details
