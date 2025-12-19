# Plugin Architecture (Future)
// turbo-all

## Philosophy

> **Core is minimal, Plugins extend functionality**

Plugins should NOT require Core code changes.

## Plugin Types

| Type | Examples |
|------|----------|
| **Feature** | Payments, Notifications, File Storage |
| **Integration** | OAuth, Webhooks, SMS Gateway |
| **UI** | Dashboard widgets, Custom pages |

## Plugin Structure

```
plugins/
└── payments/
    ├── manifest.json     # Metadata, dependencies
    ├── migrations/       # SQL migrations
    ├── src/
    │   ├── domain/
    │   ├── application/
    │   └── interface/
    └── wasm/             # WASM bundle (optional)
```

## Manifest Example

```json
{
  "name": "payments",
  "version": "1.0.0",
  "author": "Kyx Team",
  "dependencies": ["core >= 1.0"],
  "tables": ["plugin_payments", "plugin_transactions"],
  "routes": [
    { "method": "POST", "path": "/payments/charge" }
  ],
  "permissions": ["payment.create", "payment.refund"]
}
```

## Plugin Conventions

1. **Table Prefix**: `plugin_{name}_` (e.g., `plugin_payments_transactions`)
2. **Scope Isolation**: Can only access own tables + Core read-only
3. **Permission Prefix**: `{plugin_name}.*` (e.g., `payments.create`)
4. **Event Hooks**: Subscribe to Core events (user.created, etc.)

## Core Extension Points

| Hook | Description |
|------|-------------|
| `on_user_created` | After user registration |
| `on_tenant_created` | After tenant creation |
| `on_login` | Successful login |
| `on_request` | Every HTTP request |
