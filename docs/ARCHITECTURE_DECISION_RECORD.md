# Kyx Architecture Decision Record (ADR)

> **Date:** 2026-01-07  
> **Status:** Draft  
> **Authors:** Kyx Engineering Team

---

## 1. Executive Summary

This document captures architectural decisions and future roadmap based on analysis of the current kyx-kernel implementation. Key themes: **minimal core, optional plugins, graceful degradation**.

---

## 2. Core Principles

| Principle                  | Description                                          |
| -------------------------- | ---------------------------------------------------- |
| **Minimal Core**           | Only required features in kernel (~10K lines target) |
| **Optional Plugins**       | All business features as plugins                     |
| **Graceful Degradation**   | System works 100% without any plugin                 |
| **No Hard Dependencies**   | Core never crashes if plugin missing                 |
| **Database-Driven Config** | No hardcoded limits or values                        |

---

## 3. Current State (Completed)

### Phase 1-4: Signal Auth Integration ✅

| Phase | Deliverable            | Status  |
| ----- | ---------------------- | ------- |
| 1     | Signal Auth Spec (RFC) | ✅ Done |
| 2     | JWT Claims Extension   | ✅ Done |
| 3     | PlanAwareRateLimit     | ✅ Done |
| 4     | AI Governance Docs     | ✅ Done |

**Files Added:** ~1,400 lines across 14 files  
**E2E Tests:** 52/52 PASS

---

## 4. Identified Issues

### 4.1 Hardcoded Values (Must Fix)

| Location                      | Current                     | Should Be              |
| ----------------------------- | --------------------------- | ---------------------- |
| `jwt.rs` PlanFeatures         | `free/pro/enterprise` match | Database `sys_plans`   |
| `rate_limit.rs`               | Fixed limits per plan       | Read from plan config  |
| `entity.rs` PermissionMapping | Hardcoded match             | Database mapping table |
| Token TTLs                    | 30min/24h fixed             | ConfigService dynamic  |

### 4.2 Core Bloat (Should Fix)

| Component            | Current Location      | Proposed            |
| -------------------- | --------------------- | ------------------- |
| PlanFeatures struct  | `jwt.rs` (core)       | `kyx-plan-plugin`   |
| Signal module        | `src/modules/signal/` | `kyx-signal-plugin` |
| FaceSearch (planned) | N/A                   | `kyx-face-plugin`   |

---

## 5. Proposed Architecture

### 5.1 Plugin Categories

```
┌─────────────────────────────────────────────┐
│              KYX-KERNEL (Core)              │
│  Auth, RBAC, Tenant, Plugin Engine          │
│  Size: ~10,000 lines                        │
└─────────────────────────────────────────────┘
                    │
    ┌───────────────┼───────────────┐
    ▼               ▼               ▼
┌─────────┐   ┌─────────┐   ┌─────────┐
│  Plan   │   │ Signal  │   │   AI    │
│ Plugin  │   │ Plugin  │   │ Plugin  │
│(billing)│   │(realtime│   │(face,   │
│         │   │ + notif)│   │ search) │
└─────────┘   └─────────┘   └─────────┘
```

### 5.2 Service Locations

| Service             | Location                  | Notes                         |
| ------------------- | ------------------------- | ----------------------------- |
| NotificationService | **kyx-signal** (exists)   | PushService + EventDispatcher |
| PlanService         | **kyx-plan-plugin** (new) | Database-driven custom plans  |
| FaceSearchService   | **kyx-face-plugin** (new) | AI with consent governance    |

---

## 6. Future Roadmap

### Phase A: Database-Driven Plans (P1)

- [ ] Create `sys_plans` table (tenant custom plans)
- [ ] Create `PlanService` to read from DB
- [ ] Remove hardcoded `for_plan()` match
- [ ] Claims reference `plan_id` instead of `plan_type`

### Phase B: Plugin Architecture Refactor (P2)

- [ ] Move PlanFeatures to kyx-plan-plugin
- [ ] Move Signal module to kyx-signal-plugin
- [ ] Implement Plugin Provider pattern
- [ ] Test graceful degradation (no plugin = no limits)

### Phase C: Notification Integration (P1)

- [ ] Create NotificationClient in Kernel (thin HTTP client)
- [ ] Kernel → Signal API for notifications
- [ ] WebSocket online / Push offline strategy

### Phase D: AI Face Plugin (P3)

- [ ] Create kyx-face-plugin based on governance doc
- [ ] Consent management
- [ ] Privacy-compliant face encoding

---

## 7. Use Case Compatibility

| Use Case             | Plugins Needed      |
| -------------------- | ------------------- |
| Simple internal tool | None                |
| Self-hosted CMS      | None                |
| SaaS with billing    | kyx-plan            |
| Realtime chat app    | kyx-signal          |
| Enterprise w/ AI     | kyx-plan + kyx-face |
| Full platform        | All plugins         |

---

## 8. Existing kyx-signal Services

| Service                   | Purpose                              |
| ------------------------- | ------------------------------------ |
| **PushService**           | FCM/APNS token management + dispatch |
| **EventDispatcher**       | WebSocket message routing            |
| **SessionManager**        | Connection management                |
| `dispatch_notification()` | Unified notification dispatch        |
| `dispatch_to_user()`      | Target specific user sessions        |
| `dispatch_to_tenant()`    | Broadcast to tenant                  |

---

## 9. Key Decisions

| Decision                            | Rationale                                   |
| ----------------------------------- | ------------------------------------------- |
| Plan as optional plugin             | Not all deployments need billing            |
| NotificationService stays in Signal | Already implemented, separation of concerns |
| Core Claims remain minimal          | Only auth-essential fields                  |
| Database-driven plan config         | Custom tenant plans requirement             |
| Graceful degradation mandatory      | Production stability                        |

---

## 10. Next Actions

1. **Immediate:** Document sync to kyx-governance ✅
2. **Short-term:** Database-driven plans implementation
3. **Medium-term:** Plugin refactoring
4. **Long-term:** kyx-face-plugin development

---

> **Document Version:** 1.0.0  
> **Last Updated:** 2026-01-07
