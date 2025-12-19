---
description: Initial System Setup & Onboarding Workflow
---

# Initial System Setup Workflow

This workflow guides you through the process of initializing a fresh Kyx Kernel instance and setting up the first Administrative account.

## Prerequisites
- Kyx Kernel is running and connected to PostgreSQL and Redis.
- `ENGINE_SECRET_KEY` is set in your environment.

## Steps

1. **Verify Setup Status**
   Check if the system requires initialization:
   ```bash
   curl http://localhost:8080/api/v1/system/setup/status
   ```
   Should return `{"is_setup": false}`.

2. **Initialize via Platform (Recommended)**
   - Open Kyx Platform in your browser.
   - Navigate to `/setup`.
   - Enter your Engine URL and the `ENGINE_SECRET_KEY`.
   - Provide Admin details (Email, Name, Password).
   - Provide Organization Name.
   - Click "Initialize System".

3. **Verify Admin Access**
   - Log in via `/login` with your new credentials.
   - Verify that you have the `Admin` role and a valid `tenant_id`.

4. **Security Check**
   Verify that the setup endpoint is now locked:
   ```bash
   curl -X POST http://localhost:8080/api/v1/auth/setup ...
   ```
   Should return `403 Forbidden`.
