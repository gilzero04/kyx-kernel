# Kyx Plugin Specification: Analytics Plugin

> **Plugin ID:** `kyx-analytics`  
> **Version:** 1.0.0  
> **Status:** Draft  
> **Date:** 2026-01-07

---

## 1. Overview

Business intelligence and analytics plugin for tracking user behavior, conversions, and platform metrics.

### Dependencies

- **Required:** None (fully independent)
- **Optional:** All plugins (can track events from any)

---

## 2. Features

| Feature                 | Description                    |
| ----------------------- | ------------------------------ |
| **Event Tracking**      | Page views, clicks, actions    |
| **Conversion Funnels**  | Multi-step conversion tracking |
| **Cohort Analysis**     | User cohort grouping           |
| **Real-time Dashboard** | Live metrics                   |
| **Export**              | CSV, JSON data export          |

---

## 3. Database Schema

```sql
-- Events
CREATE TABLE plugin_analytics_events (
  id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  user_id UUID,
  session_id VARCHAR(100),
  event_name VARCHAR(100) NOT NULL,
  event_category VARCHAR(50),
  properties JSONB DEFAULT '{}',
  page_url TEXT,
  referrer TEXT,
  user_agent TEXT,
  ip_hash VARCHAR(64),
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Daily aggregates
CREATE TABLE plugin_analytics_daily (
  id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  date DATE NOT NULL,
  metric_name VARCHAR(100) NOT NULL,
  metric_value DECIMAL(20,4),
  dimensions JSONB DEFAULT '{}',
  UNIQUE(tenant_id, date, metric_name, dimensions)
);

-- Funnels
CREATE TABLE plugin_analytics_funnels (
  id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  name VARCHAR(200) NOT NULL,
  steps JSONB NOT NULL,
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 4. API Endpoints

| Method         | Endpoint                              | Permission         | Description    |
| -------------- | ------------------------------------- | ------------------ | -------------- |
| **Tracking**   |
| POST           | `/api/v1/analytics/track`             | (auth)             | Track event    |
| POST           | `/api/v1/analytics/page`              | (auth)             | Track pageview |
| **Reports**    |
| GET            | `/api/v1/admin/analytics/overview`    | `analytics:read`   | Dashboard data |
| GET            | `/api/v1/admin/analytics/events`      | `analytics:read`   | Event list     |
| GET            | `/api/v1/admin/analytics/funnel/{id}` | `analytics:read`   | Funnel report  |
| **Management** |
| POST           | `/api/v1/admin/analytics/funnels`     | `analytics:manage` | Create funnel  |
| GET            | `/api/v1/admin/analytics/export`      | `analytics:export` | Export data    |

---

## 5. Event Types

```yaml
# Auto-tracked by plugin
auto_events:
  - session.start
  - session.end
  - page.view
  - user.signup
  - user.login

# Available for integration
integration_events:
  - plan.upgrade (from kyx-plan)
  - affiliate.conversion (from kyx-affiliate)
  - notification.clicked (from kyx-signal)
  - face.enrolled (from kyx-face)
```

---

## 6. Manifest

```yaml
id: kyx-analytics
name: "Kyx Analytics"
version: "1.0.0"
author: "Kyx Team"
description: "Business intelligence and tracking"
capabilities:
  - database:read
  - database:write
  - event:subscribe
permissions:
  - analytics:read
  - analytics:manage
  - analytics:export
settings:
  retention_days: 365
  sample_rate: 1.0
  ip_anonymization: true
optional_integrations:
  - kyx-plan
  - kyx-affiliate
  - kyx-signal
  - kyx-face
```
