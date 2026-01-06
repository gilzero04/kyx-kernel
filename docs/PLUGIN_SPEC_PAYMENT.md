# Kyx Plugin Specification: Payment Plugin

> **Plugin ID:** `kyx-payment`  
> **Version:** 1.0.0  
> **Status:** Draft  
> **Date:** 2026-01-07

---

## 1. Overview

Payment processing plugin supporting multiple gateways (Stripe, PayPal, Omise). Independent but integrates with Plan and Affiliate.

### Dependencies

- **Required:** None (can accept one-time payments)
- **Optional:** `kyx-plan` (for subscriptions)
- **Optional:** `kyx-affiliate` (for commission tracking)

---

## 2. Features

| Feature               | Description                 |
| --------------------- | --------------------------- |
| **Multi-Gateway**     | Stripe, PayPal, Omise, etc. |
| **One-time Payments** | Single purchases            |
| **Subscriptions**     | Recurring billing           |
| **Invoicing**         | Auto-generate invoices      |
| **Refunds**           | Process refunds             |
| **Webhooks**          | Gateway event handling      |

---

## 3. Database Schema

```sql
-- Payment methods
CREATE TABLE plugin_payment_methods (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  gateway VARCHAR(50) NOT NULL,
  gateway_customer_id VARCHAR(200),
  gateway_method_id VARCHAR(200),
  type VARCHAR(20),  -- card, bank, wallet
  last_four VARCHAR(4),
  brand VARCHAR(20),
  is_default BOOLEAN DEFAULT false,
  expires_at DATE,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Transactions
CREATE TABLE plugin_transactions (
  id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  user_id UUID NOT NULL,
  gateway VARCHAR(50) NOT NULL,
  gateway_transaction_id VARCHAR(200),
  type VARCHAR(20),  -- charge, refund, subscription
  amount DECIMAL(12,2) NOT NULL,
  currency VARCHAR(3) DEFAULT 'USD',
  status VARCHAR(20) DEFAULT 'pending',
  metadata JSONB DEFAULT '{}',
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Subscriptions (if kyx-plan integrated)
CREATE TABLE plugin_subscriptions (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  plan_id UUID,  -- Optional: from kyx-plan
  gateway VARCHAR(50) NOT NULL,
  gateway_subscription_id VARCHAR(200),
  status VARCHAR(20) DEFAULT 'active',
  current_period_start TIMESTAMPTZ,
  current_period_end TIMESTAMPTZ,
  cancel_at_period_end BOOLEAN DEFAULT false,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Invoices
CREATE TABLE plugin_invoices (
  id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  user_id UUID NOT NULL,
  subscription_id UUID REFERENCES plugin_subscriptions(id),
  invoice_number VARCHAR(50) UNIQUE,
  amount DECIMAL(12,2) NOT NULL,
  currency VARCHAR(3) DEFAULT 'USD',
  status VARCHAR(20) DEFAULT 'draft',
  due_date DATE,
  paid_at TIMESTAMPTZ,
  pdf_url TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 4. API Endpoints

| Method              | Endpoint                          | Permission       | Description         |
| ------------------- | --------------------------------- | ---------------- | ------------------- |
| **Payment Methods** |
| GET                 | `/api/v1/me/payment-methods`      | (self)           | List my methods     |
| POST                | `/api/v1/me/payment-methods`      | (self)           | Add method          |
| DELETE              | `/api/v1/me/payment-methods/{id}` | (self)           | Remove method       |
| **Checkout**        |
| POST                | `/api/v1/payment/checkout`        | (auth)           | Create checkout     |
| POST                | `/api/v1/payment/confirm`         | (auth)           | Confirm payment     |
| **Subscriptions**   |
| POST                | `/api/v1/me/subscription`         | (self)           | Subscribe to plan   |
| DELETE              | `/api/v1/me/subscription`         | (self)           | Cancel subscription |
| **Admin**           |
| GET                 | `/api/v1/admin/transactions`      | `payment:read`   | List transactions   |
| POST                | `/api/v1/admin/refund`            | `payment:refund` | Process refund      |
| **Webhooks**        |
| POST                | `/api/v1/webhooks/stripe`         | (system)         | Stripe webhook      |
| POST                | `/api/v1/webhooks/paypal`         | (system)         | PayPal webhook      |

---

## 5. Integration Hooks

```rust
// Called after successful payment
fn on_payment_success(transaction: &Transaction) {
    // Notify affiliate plugin if available
    if let Some(affiliate) = plugins.get("kyx-affiliate") {
        affiliate.on_payment(transaction.user_id, transaction.amount, "subscription");
    }

    // Notify analytics if available
    if let Some(analytics) = plugins.get("kyx-analytics") {
        analytics.track("payment.success", transaction.metadata);
    }
}

// Called on subscription change
fn on_subscription_changed(sub: &Subscription) {
    // Update plan assignment if kyx-plan available
    if let Some(plan) = plugins.get("kyx-plan") {
        plan.update_user_plan(sub.user_id, sub.plan_id);
    }
}
```

---

## 6. Graceful Degradation

| Scenario                    | Behavior                |
| --------------------------- | ----------------------- |
| Plugin not installed        | No payment features     |
| kyx-plan not installed      | One-time payments only  |
| kyx-affiliate not installed | No commission tracking  |
| Gateway down                | Return 503, retry queue |

---

## 7. Manifest

```yaml
id: kyx-payment
name: "Kyx Payment Processing"
version: "1.0.0"
author: "Kyx Team"
description: "Multi-gateway payment and subscriptions"
capabilities:
  - database:read
  - database:write
  - http:external
  - webhook:receive
permissions:
  - payment:read
  - payment:charge
  - payment:refund
  - subscription:manage
settings:
  default_gateway: "stripe"
  webhook_secret: "${STRIPE_WEBHOOK_SECRET}"
  invoice_prefix: "INV-"
optional_integrations:
  - kyx-plan # For subscription plans
  - kyx-affiliate # For commission tracking
  - kyx-analytics # For conversion tracking
```
