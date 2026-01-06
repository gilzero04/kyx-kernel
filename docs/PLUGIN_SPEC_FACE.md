# Kyx Plugin Specification: Face Recognition Plugin

> **Plugin ID:** `kyx-face`  
> **Version:** 1.0.0  
> **Status:** Draft  
> **Date:** 2026-01-07

---

## 1. Overview

AI-powered face recognition plugin with privacy-first design. Supports face enrollment, search, and matching with consent management.

### Dependencies

- **Required:** `kyx-plan` (for AI quotas - Pro/Enterprise only)
- **Required:** External face recognition service
- **Governance:** [AI_GOVERNANCE_FACE_SEARCH.md](AI_GOVERNANCE_FACE_SEARCH.md)

---

## 2. Features

| Feature                | Description                      |
| ---------------------- | -------------------------------- |
| **Face Enrollment**    | Encode and store face embeddings |
| **Face Search**        | 1:N matching against database    |
| **Consent Management** | PDPA/GDPR compliant consent      |
| **Auto Purge**         | Automatic data retention         |
| **Audit Logging**      | Full operation history           |

---

## 3. Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     KYX-KERNEL                          │
│  ┌─────────────────────────────────────────────────────┐│
│  │              kyx-face-plugin                        ││
│  │  - Consent management                               ││
│  │  - Quota enforcement (via kyx-plan)                 ││
│  │  - Audit logging                                    ││
│  └─────────────────────┬───────────────────────────────┘│
└────────────────────────┼────────────────────────────────┘
                         │ gRPC/HTTP
                         ▼
┌─────────────────────────────────────────────────────────┐
│                  FACE RECOGNITION SERVICE               │
│  InsightFace / FaceNet / custom model                  │
│  Embedding generation, vector search                    │
└─────────────────────────────────────────────────────────┘
```

---

## 4. Database Schema

```sql
-- Consent records
CREATE TABLE plugin_face_consents (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  purpose VARCHAR(50) NOT NULL,
  scope TEXT[] DEFAULT '{}',
  granted_at TIMESTAMPTZ DEFAULT NOW(),
  expires_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  is_active BOOLEAN DEFAULT true
);

-- Face embeddings (encrypted)
CREATE TABLE plugin_face_embeddings (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  consent_id UUID REFERENCES plugin_face_consents(id),
  embedding_encrypted BYTEA NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  expires_at TIMESTAMPTZ NOT NULL
);

-- Audit log
CREATE TABLE plugin_face_audit (
  id UUID PRIMARY KEY,
  actor_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  action VARCHAR(50) NOT NULL,
  target_user_id UUID,
  details JSONB,
  ip_address VARCHAR(45),
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 5. API Endpoints

| Method              | Endpoint                      | Permission       | Description         |
| ------------------- | ----------------------------- | ---------------- | ------------------- |
| **Consent**         |
| POST                | `/api/v1/ai/face/consent`     | (self)           | Grant consent       |
| GET                 | `/api/v1/ai/face/consent`     | (self)           | Get consent status  |
| DELETE              | `/api/v1/ai/face/consent`     | (self)           | Revoke consent      |
| **Face Operations** |
| POST                | `/api/v1/ai/face/enroll`      | `ai:face:enroll` | Enroll face         |
| POST                | `/api/v1/ai/face/search`      | `ai:face:search` | Search faces        |
| DELETE              | `/api/v1/ai/face/{user_id}`   | `ai:face:delete` | Delete face data    |
| **Admin**           |
| GET                 | `/api/v1/admin/ai/face/audit` | `ai:face:audit`  | View audit logs     |
| POST                | `/api/v1/admin/ai/face/purge` | `ai:face:delete` | Force purge expired |

---

## 6. Plan-Based Quotas (Required)

| Plan       | Enrollments/day | Searches/min | Storage |
| ---------- | --------------- | ------------ | ------- |
| free       | ❌ Disabled     | ❌ Disabled  | ❌      |
| pro        | 100             | 20           | 1,000   |
| enterprise | 10,000          | 200          | 100,000 |

---

## 7. Plugin Hooks

```rust
// Check consent before any operation
fn require_consent(user_id: Uuid, purpose: &str) -> Result<ConsentId>;

// Called before face enrollment
fn on_enroll(user_id: Uuid, image: &[u8]) -> Result<EmbeddingId>;

// Called on face search
fn on_search(query_image: &[u8], threshold: f32) -> Result<Vec<Match>>;

// Audit logging
fn audit_log(action: &str, actor: Uuid, details: Value);
```

---

## 8. Privacy Controls

### Encryption

```rust
// All embeddings encrypted at rest
let encrypted = encrypt_aes256_gcm(embedding, tenant_key);

// Keys managed in Vault/HSM
let tenant_key = vault.get_key(tenant_id);
```

### Retention

```rust
// Auto-purge job (daily)
DELETE FROM plugin_face_embeddings
WHERE expires_at < NOW() OR consent_revoked = true;
```

---

## 9. Graceful Degradation

| Scenario               | Behavior                        |
| ---------------------- | ------------------------------- |
| Plugin not installed   | Face endpoints return 404       |
| kyx-plan not installed | Face operations disabled        |
| Plan = free            | Face operations disabled (403)  |
| Face service down      | Return 503, queue retryable ops |
| Consent revoked        | Delete data, block new ops      |

---

## 10. Manifest

```yaml
id: kyx-face
name: "Kyx Face Recognition"
version: "1.0.0"
author: "Kyx Team"
description: "Privacy-first face recognition with consent"
capabilities:
  - database:read
  - database:write
  - http:external
  - encryption:use
permissions:
  - ai:face:enroll
  - ai:face:search
  - ai:face:delete
  - ai:face:audit
settings:
  face_service_url: "${FACE_SERVICE_URL}"
  default_retention_days: 90
  encryption_key_rotation_days: 90
required_integrations:
  - kyx-plan # AI features require Pro+ plan
```

---

## 11. Compliance

| Regulation  | Requirement           | Implementation          |
| ----------- | --------------------- | ----------------------- |
| PDPA Art 26 | Consent for biometric | ConsentService          |
| GDPR Art 9  | Special category data | Encryption + consent    |
| GDPR Art 17 | Right to erasure      | Revoke consent = delete |
| GDPR Art 30 | Processing records    | Audit logging           |
