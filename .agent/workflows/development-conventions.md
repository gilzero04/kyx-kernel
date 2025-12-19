# Development Conventions
// turbo-all

## Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| **Tables** | `snake_case`, prefixed | `auth_users`, `sys_roles` |
| **Columns** | `snake_case` | `created_at`, `tenant_type_id` |
| **Rust Structs** | `PascalCase` | `AuthService`, `UserRole` |
| **Rust Functions** | `snake_case` | `create_user`, `verify_token` |
| **API Endpoints** | `kebab-case` | `/setup/verify-key` |
| **API Scopes** | `/api/v1/{module}` | `/api/v1/auth/login` |

## Table Prefix Standards

| Prefix | Purpose | Example |
|--------|---------|---------|
| `auth_` | Authentication entities | `auth_users`, `auth_tenants` |
| `sys_` | System/config tables | `sys_roles`, `sys_permissions` |
| `audit_` | Audit logging | `audit_logs` |
| `plugin_` | Plugin-specific tables | `plugin_payments` |

## Timestamp Fields

| Table Type | created_at | updated_at | deleted_at |
|------------|:----------:|:----------:|:----------:|
| **Entity** (users, tenants) | ✅ | ✅ | ✅ |
| **System** (roles, permissions) | ✅ | ✅ | ✅ |
| **Join** (memberships) | ✅ | ❌ | ❌ |
| **Transaction** (audit_logs) | ✅ | ❌ | ❌ |

## Git Commit Format

```
<type>(<scope>): <description>

Types: feat, fix, docs, style, refactor, test, chore
Scope: auth, system, core, platform, db
```

## Code Organization

```
src/modules/{module}/
├── domain/         # Entities, Value Objects
├── application/    # Services, Use Cases
├── infrastructure/ # DB queries, external APIs
└── interface/
    └── http/       # HTTP handlers (routes)
```
