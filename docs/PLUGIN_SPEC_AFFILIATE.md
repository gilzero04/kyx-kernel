# Kyx Plugin Specification: Affiliate Plugin

> **Plugin ID:** `kyx-affiliate`  
> **Version:** 1.0.0  
> **Status:** Draft  
> **Date:** 2026-01-07

---

## 1. Overview

Optional affiliate/referral marketing plugin. Works independently - **does not require kyx-plan plugin**.

### Dependencies

- **Required:** None (fully independent)
- **Optional:** `kyx-plan` (for plan-based commission rates)

---

## 2. Features

| Feature                   | Description                     |
| ------------------------- | ------------------------------- |
| **Referral Codes**        | Unique codes per affiliate      |
| **Multi-tier Commission** | Level 1, 2, 3 referrals         |
| **Cookie Attribution**    | 30/60/90 day tracking           |
| **Payout Management**     | Track pending/paid              |
| **Custom Rates**          | Per-affiliate or per-plan rates |
| **Analytics Dashboard**   | Clicks, conversions, revenue    |

---

## 3. Database Schema

```sql
-- Affiliates (users who can refer)
CREATE TABLE plugin_affiliates (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL UNIQUE,
  tenant_id UUID NOT NULL,
  code VARCHAR(50) UNIQUE NOT NULL,
  commission_rate DECIMAL(5,2) DEFAULT 10.00,  -- percentage
  tier2_rate DECIMAL(5,2) DEFAULT 5.00,
  tier3_rate DECIMAL(5,2) DEFAULT 2.00,
  cookie_days INT DEFAULT 30,
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Referrals (who was referred by whom)
CREATE TABLE plugin_referrals (
  id UUID PRIMARY KEY,
  affiliate_id UUID REFERENCES plugin_affiliates(id),
  referred_user_id UUID NOT NULL UNIQUE,
  referred_tenant_id UUID,
  source VARCHAR(50),  -- link, email, social
  cookie_set_at TIMESTAMPTZ,
  converted_at TIMESTAMPTZ,
  status VARCHAR(20) DEFAULT 'pending',  -- pending, converted, expired
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Commissions earned
CREATE TABLE plugin_commissions (
  id UUID PRIMARY KEY,
  affiliate_id UUID REFERENCES plugin_affiliates(id),
  referral_id UUID REFERENCES plugin_referrals(id),
  tier INT DEFAULT 1,  -- 1, 2, or 3
  amount DECIMAL(12,2) NOT NULL,
  currency VARCHAR(3) DEFAULT 'USD',
  source_event VARCHAR(50),  -- signup, purchase, subscription
  source_amount DECIMAL(12,2),
  status VARCHAR(20) DEFAULT 'pending',  -- pending, approved, paid, rejected
  approved_at TIMESTAMPTZ,
  paid_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Payouts
CREATE TABLE plugin_payouts (
  id UUID PRIMARY KEY,
  affiliate_id UUID REFERENCES plugin_affiliates(id),
  amount DECIMAL(12,2) NOT NULL,
  currency VARCHAR(3) DEFAULT 'USD',
  method VARCHAR(50),  -- bank, paypal, crypto
  reference VARCHAR(200),
  status VARCHAR(20) DEFAULT 'pending',
  processed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Click tracking
CREATE TABLE plugin_affiliate_clicks (
  id UUID PRIMARY KEY,
  affiliate_id UUID REFERENCES plugin_affiliates(id),
  ip_hash VARCHAR(64),
  user_agent TEXT,
  referrer TEXT,
  landing_page TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 4. API Endpoints

| Method               | Endpoint                                 | Permission           | Description                |
| -------------------- | ---------------------------------------- | -------------------- | -------------------------- |
| **Public**           |
| GET                  | `/api/v1/affiliate/join`                 | (auth)               | Apply to become affiliate  |
| GET                  | `/ref/{code}`                            | (public)             | Referral redirect + cookie |
| **Affiliate Portal** |
| GET                  | `/api/v1/affiliate/me`                   | (affiliate)          | My affiliate profile       |
| GET                  | `/api/v1/affiliate/me/referrals`         | (affiliate)          | My referrals               |
| GET                  | `/api/v1/affiliate/me/commissions`       | (affiliate)          | My earnings                |
| GET                  | `/api/v1/affiliate/me/payouts`           | (affiliate)          | Payout history             |
| POST                 | `/api/v1/affiliate/me/payout-request`    | (affiliate)          | Request payout             |
| **Admin**            |
| GET                  | `/api/v1/admin/affiliates`               | `affiliate:read`     | List affiliates            |
| POST                 | `/api/v1/admin/affiliates`               | `affiliate:create`   | Create affiliate           |
| PATCH                | `/api/v1/admin/affiliates/{id}`          | `affiliate:update`   | Update rates               |
| POST                 | `/api/v1/admin/affiliates/{id}/approve`  | `affiliate:approve`  | Approve affiliate          |
| GET                  | `/api/v1/admin/commissions`              | `commission:read`    | All commissions            |
| POST                 | `/api/v1/admin/commissions/{id}/approve` | `commission:approve` | Approve commission         |
| POST                 | `/api/v1/admin/payouts/{id}/process`     | `payout:process`     | Process payout             |

---

## 5. Commission Events

| Event                  | Source            | Commission Applied      |
| ---------------------- | ----------------- | ----------------------- |
| `user.signup`          | Via referral link | ✅ If configured        |
| `subscription.created` | Purchase          | ✅ Based on amount      |
| `payment.received`     | Recurring         | ✅ Recurring if enabled |

---

## 6. Plugin Hooks

```rust
// Track referral click
fn on_referral_click(code: &str, request: &Request);

// Called when user signs up with referral cookie
fn on_user_signup(user_id: Uuid, referral_code: Option<String>);

// Called on payment (optional: integrate with payment plugin)
fn on_payment(user_id: Uuid, amount: Decimal, event: &str);

// Calculate commission (can integrate with kyx-plan if available)
fn calculate_commission(affiliate_id: Uuid, amount: Decimal) -> Decimal;
```

---

## 7. Integration with kyx-plan (Optional)

```rust
// If kyx-plan is installed, use plan-based rates
if let Some(plan_plugin) = plugins.get("kyx-plan") {
    let plan = plan_plugin.get_user_plan(user_id);
    commission_rate = match plan.name.as_str() {
        "pro" => 15.0,        // Higher rate for Pro referrals
        "enterprise" => 20.0,  // Highest for Enterprise
        _ => 10.0,            // Default
    };
} else {
    // No plan plugin - use affiliate's custom rate
    commission_rate = affiliate.commission_rate;
}
```

---

## 8. Graceful Degradation

| Scenario               | Behavior                        |
| ---------------------- | ------------------------------- |
| Plugin not installed   | Referral links 404, no tracking |
| kyx-plan not installed | Fixed rates per affiliate       |
| DB error               | Log error, skip commission      |

---

## 9. Cookie Strategy

```
GET /ref/ABC123
  → Set cookie: kyx_ref=ABC123; Max-Age=2592000; HttpOnly
  → Redirect to landing page

On signup:
  → Check cookie kyx_ref
  → If exists and valid, create referral
```

---

## 10. Manifest

```yaml
id: kyx-affiliate
name: "Kyx Affiliate Program"
version: "1.0.0"
author: "Kyx Team"
description: "Referral and affiliate marketing"
capabilities:
  - database:read
  - database:write
  - cookie:write
  - event:subscribe
permissions:
  - affiliate:read
  - affiliate:create
  - affiliate:update
  - affiliate:approve
  - commission:read
  - commission:approve
  - payout:read
  - payout:process
settings:
  default_commission_rate: 10.0
  cookie_duration_days: 30
  min_payout_amount: 50.0
  auto_approve_threshold: 100.0
optional_integrations:
  - kyx-plan # For plan-based commission rates
```

---

## 11. Use Cases

| Use Case                     | Plugins Required                     |
| ---------------------------- | ------------------------------------ |
| Simple referral program      | kyx-affiliate only                   |
| SaaS with tiered commissions | kyx-affiliate + kyx-plan             |
| E-commerce affiliate         | kyx-affiliate + payment integration  |
| Multi-tier MLM style         | kyx-affiliate (tier2, tier3 enabled) |
