# API Standards
// turbo-all

## Base URL

```
Development: http://localhost:8080/api/v1
Production:  https://api.kyx.tech/api/v1
```

## Response Format

### Success
```json
{
  "data": { ... },
  "meta": { "page": 1, "total": 100 }
}
```

### Error
```json
{
  "status": "error",
  "message": "Human readable message",
  "code": "ERROR_CODE"
}
```

## HTTP Status Codes

| Code | Meaning | When to Use |
|:----:|---------|-------------|
| 200 | OK | Successful GET, PATCH |
| 201 | Created | Successful POST (new resource) |
| 204 | No Content | Successful DELETE |
| 400 | Bad Request | Validation errors |
| 401 | Unauthorized | Missing/invalid token |
| 403 | Forbidden | Valid token but no permission |
| 404 | Not Found | Resource doesn't exist |
| 409 | Conflict | Duplicate entry |
| 500 | Server Error | Unexpected errors |

## Authentication

### Headers
```
Authorization: Bearer <access_token>
X-Engine-Secret: <engine_key>  # For setup/internal only
```

### Endpoints Protection

| Scope | Requires | Example |
|-------|----------|---------|
| `/api/v1/auth/login` | None | Public |
| `/api/v1/auth/setup` | X-Engine-Secret | Setup only |
| `/api/v1/system/*` | Bearer Token | Protected |
| `/internal/*` | X-Engine-Secret | Internal |

## Pagination

```
GET /users?page=1&limit=20
```

Response includes `meta.page`, `meta.limit`, `meta.total`
