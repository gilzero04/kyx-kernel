---
description: RBAC Standards and Permission Conventions
---

# RBAC Standards

This document defines the naming conventions and structure for Role-Based Access Control (RBAC) in the Kyx platform.

## 1. Permission Naming

Permissions follow the pattern `resource:action`.

### Actions

- `read`: View records and details.
- `create`: Add new records.
- `update`: Modify existing records.
- `delete`: Deactivate or archive records (Soft-delete).
- `manage`: Full control over a resource (implicit read/create/update/delete).

### Standard Modules

- `user:*`: User management.
- `tenant:*`: Multi-tenancy and organization settings.
- `rbac:*`: Roles and permissions management.
- `audit:*`: Activity logs and history.
- `system:*`: Core kernel settings (Only for System Owner).

## 2. Global vs Scoped Permissions

- **System Permissions (`is_system = TRUE`)**: Reserved for the platform owner. Cannot be delegated to sub-tenants.
- **Tenant Permissions (`is_system = FALSE`)**: Can be delegated from a parent tenant to a child tenant.

## 3. Delegation Rules

- A tenant can only grant permissions to its roles that are present in the `sys_tenant_permissions` table (White-list).
- When creating a child tenant, the parent can only delegate a subset of its own permissions to the child.
- System permissions are **never** delegated downstream.

## 4. Role Isolation

- Roles are scoped to a `tenant_id`.
- Global roles (e.g. `superadmin`) can exist without a `tenant_id` if they are system-wide.
- Slugs for tenant-scoped roles must be unique within that tenant.

## 5. Role Sharing (RULE 19)

- **Broadcast Sharing**: Set `is_shared = TRUE` to share role with ALL descendant tenants.
- **Explicit Sharing**: Use `sys_resource_shares` for sharing to specific tenants.
- **Query Pattern**: Roles use `can_access_shared_resource('role', id, $tenant_id)` to include shared roles.

## 6. Implementation Pattern

// turbo

### Backend Check

```rust
if claims.permissions.contains("user:create") {
    // Authorized
}
```

### Frontend Check

```javascript
{#if $auth.hasPermission('user:create')}
  <Button on:click={showAddUserModal}>Add User</Button>
{/if}
```
