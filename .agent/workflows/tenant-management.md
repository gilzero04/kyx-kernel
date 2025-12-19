# Tenant Management Workflow
// turbo-all

This workflow describes how to manage tenants in the Kyx Kernel hierarchical system.

## Tenant Hierarchy

```
Owner (Global SuperAdmin)
├── Tenant A (HQ) ← parent_id = NULL
│   └── Tenant A1 (Branch) ← parent_id = Tenant A
└── Tenant B (HQ) ← parent_id = NULL
    └── Tenant B1 (Branch) ← parent_id = Tenant B
```

## Creating a Root Tenant (via API)

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -H "X-Engine-Secret: YOUR_ENGINE_KEY" \
  -d '{
    "email": "admin@company.com",
    "password": "securepassword",
    "full_name": "Company Admin",
    "org_name": "Company HQ"
  }'
```

## Creating a Child Tenant (Future Plugin)

```bash
curl -X POST http://localhost:8080/api/v1/tenants/{parent_id}/children \
  -H "Authorization: Bearer TOKEN" \
  -d '{
    "name": "Branch Office",
    "slug": "branch-office"
  }'
```

## Scoped Access Rules

| User | Role | Can See |
|------|------|---------|
| Owner | superadmin | All Tenants |
| Tenant Admin | superadmin | Own Tenant + Children |
| Branch Admin | superadmin | Own Branch Only |

## Database Schema

```sql
auth_tenants (
  id UUID PRIMARY KEY,
  name VARCHAR(255),
  slug VARCHAR(255) UNIQUE,
  parent_id UUID REFERENCES auth_tenants(id),  -- Hierarchy
  is_active BOOLEAN,
  created_at, updated_at, deleted_at
)
```
