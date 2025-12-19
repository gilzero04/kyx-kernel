---
description: How to manage dynamic kernel configurations safely
---

# Dynamic Configuration Management

This workflow defines how to update critical kernel settings (e.g., Token Expiry, Rate Limits) dynamically without a server restart.

## Prerequisites
- **Role**: Must have `SuperAdmin` privileges.
- **Tools**: Access to the Admin Dashboard or authorized API client.

## Steps

### 1. Identify the Setting
Locate the setting you wish to change (e.g., `refresh_token_expire_minutes`).

### 2. Verify Safety Bounds
Ensure the new value is within the allowed range defined in the kernel:
- **Refresh Token**: Max 30 days.
- **Access Token**: Max 24 hours.

### 3. Apply the Change
Perform a `PATCH` request to the Admin Config endpoint:
```bash
npx kyx-cli config set refresh_token_expire_minutes 1440
```

### 4. Verify in Audit Logs
Check the `audit_logs` table to ensure the action was recorded:
```sql
SELECT * FROM audit_logs WHERE action = 'CONFIG_UPDATE' ORDER BY timestamp DESC LIMIT 1;
```

### 5. Propagate Changes
The kernel uses a TTL-based cache. Changes will be reflected across all instances within 60 seconds.

## Safety & Security
- **Fail-Soft**: If Redis is unreachable, the kernel automatically falls back to `.env` defaults.
- **Immutability**: Certain "Hard-Locked" settings cannot be changed dynamically for kernel stability.
