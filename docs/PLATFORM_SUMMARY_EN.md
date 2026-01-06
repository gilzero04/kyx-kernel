# Kyx Platform

### Enterprise-Grade Intelligent Business Platform

---

## Overview

**Kyx Platform** is a production-ready software infrastructure designed to power modern digital businesses, from e-commerce storefronts to full-scale marketplaces.

---

## Core Differentiators

### 1. Hierarchical Multi-Tenancy

- Supports recursive organizational structures: Platform Owner → Branch → Vendor → Partner
- Independent branding and configuration at each level
- Custom domain support per tenant

### 2. Financial-Ready Architecture

- ACID-compliant transaction layer using PostgreSQL
- Isolated Ledger Interface for audit compliance
- Designed for payment integration from day one
- PostgreSQL as the single source of truth for financial data

### 3. Realtime-Native Design

- Built-in WebSocket infrastructure for chat, video, and notifications
- LiveChat with per-room message ordering and idempotent delivery
- WebRTC/LiveKit integration for video conferencing
- TTL-based presence system
- Supports thousands of concurrent connections with graceful degradation

### 4. Extensible Plugin System

- Lean kernel with optional WASM-based plugins
- Extend with wallet, payment gateway, analytics without modifying core
- Capability-based security for each plugin
- Sandboxed execution environment

### 5. AI Governance Framework

- 32 governance rules controlling AI agent behavior
- Prevents AI from making destructive changes in production
- Knowledge base with semantic search
- Incident tracking for continuous learning

---

## Technology Stack

| Component          | Technology               | Purpose                        |
| ------------------ | ------------------------ | ------------------------------ |
| **kyx-kernel**     | Rust + Ntex + PostgreSQL | Core Auth, RBAC, Multi-tenancy |
| **kyx-signal**     | Rust + Ntex + SurrealDB  | Realtime Communication         |
| **kyx-platform**   | SvelteKit + TypeScript   | Frontend Application           |
| **kyx-governance** | Rust + SurrealDB         | AI Knowledge & Rules           |
| **kyx-infra**      | Redis + Docker + TLS     | Caching, Sessions, Deployment  |

---

## Enterprise Features

| Feature                    | Status            |
| -------------------------- | ----------------- |
| Hierarchical Tenants       | ✅ Ready          |
| Custom Branding per Tenant | ✅ Ready          |
| Role-based Landing Pages   | ✅ Ready          |
| API Key Management         | ✅ Ready          |
| Audit Logging              | ✅ Ready          |
| i18n (English/Thai)        | ✅ Ready          |
| Theme System               | ✅ Ready          |
| Plugin System              | 🔄 In Development |
| Payment Gateway            | 📋 Planned        |

---

## Ideal Use Cases

### Retail & Commerce

- **E-commerce & Marketplaces**: Multi-vendor platforms with complex hierarchies
- **POS (Point of Sale)**: In-store sales with inventory and multi-branch support

### Service Industries

- **Hospital Management Systems**: Appointments, patient records, real-time notifications
- **HR Management**: Employee management, organizational hierarchy, leave management

### Financial Services

- **Finance/Fintech**: Digital wallets, transfers, financial reporting
- **SaaS Applications**: White-label solutions with tenant isolation

### Communication Platforms

- **Communication Platforms**: Built-in chat, video, and presence systems
- **AI-Powered Services**: Safe AI integration with governance guardrails

---

## Competitive Advantages

| vs Competitor | Kyx Advantage                               |
| ------------- | ------------------------------------------- |
| Shopify       | Self-hosted, true multi-tier tenancy        |
| Firebase      | PostgreSQL ACID, financial compliance       |
| Supabase      | Realtime-native design, Plugin architecture |
| Custom Build  | 80% ready, governance included              |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      kyx-platform (UI)                       │
│                        SvelteKit                             │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   kyx-kernel    │  │   kyx-signal    │  │ kyx-governance  │
│   (Core API)    │  │  (Realtime)     │  │  (AI Rules)     │
│  PostgreSQL     │  │   SurrealDB     │  │   SurrealDB     │
└─────────────────┘  └─────────────────┘  └─────────────────┘
              │               │               │
              └───────────────┼───────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      kyx-infra                               │
│              Redis · Docker · TLS · Monitoring               │
└─────────────────────────────────────────────────────────────┘
```

---

## Contact

For more information or a system demonstration, please contact the development team.

---

_This document is part of the Kyx Platform Documentation_
_Last updated: January 2026_
