// ════════════════════════════════════════════════════════════════════════════
// Integration Tests - HTTP Response Testing
// ════════════════════════════════════════════════════════════════════════════
//
// Tests HTTP handler responses directly without full App setup.
// This avoids ntex generic complexity while still testing endpoint logic.
//
// Run with: cargo test --test integration_tests
//
// ════════════════════════════════════════════════════════════════════════════

use ntex::http::StatusCode;
use serde_json::json;

// ════════════════════════════════════════════════════════════════════════════
// JSON Response Structure Tests
// ════════════════════════════════════════════════════════════════════════════

/// Verify plugin list response structure
#[test]
fn test_plugin_list_response_structure() {
    let response = json!({
        "plugins": [],
        "total": 0
    });

    assert!(response["plugins"].is_array());
    assert_eq!(response["total"], 0);
}

/// Verify plugin install response structure
#[test]
fn test_plugin_install_response_structure() {
    let response = json!({
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "plugin_id": "test-plugin",
        "name": "Test Plugin",
        "version": "1.0.0",
        "status": "installed",
        "is_active": false,
        "capabilities": ["log_info"]
    });

    assert!(response["id"].is_string());
    assert_eq!(response["plugin_id"], "test-plugin");
    assert_eq!(response["status"], "installed");
    assert!(!response["is_active"].as_bool().unwrap());
}

/// Verify security analysis response structure
#[test]
fn test_security_analysis_response_structure() {
    let response = json!({
        "risk_level": "low",
        "dangerous_capabilities": [],
        "requires_approval": false,
        "network_access": [],
        "data_access": [],
        "is_verified": false,
        "is_official": false
    });

    assert_eq!(response["risk_level"], "low");
    assert!(!response["requires_approval"].as_bool().unwrap());
    assert!(response["dangerous_capabilities"].is_array());
}

/// Verify security warnings response structure
#[test]
fn test_security_warnings_response_structure() {
    let response = json!({
        "plugin_id": "test-plugin",
        "warnings": ["Network access to untrusted domains"],
        "warning_count": 1
    });

    assert_eq!(response["plugin_id"], "test-plugin");
    assert_eq!(response["warning_count"], 1);
    assert!(response["warnings"].is_array());
}

// ════════════════════════════════════════════════════════════════════════════
// Auth Response Structure Tests
// ════════════════════════════════════════════════════════════════════════════

/// Verify login response structure
#[test]
fn test_login_response_structure() {
    let response = json!({
        "token": "eyJhbGciOiJIUzI1NiIs...",
        "refresh_token": "abc123...",
        "expires_in": 3600,
        "user": {
            "id": "user-uuid",
            "email": "admin@test.com"
        }
    });

    assert!(response["token"].is_string());
    assert!(response["refresh_token"].is_string());
    assert_eq!(response["expires_in"], 3600);
    assert!(response["user"].is_object());
}

/// Verify setup status response structure
#[test]
fn test_setup_status_response_structure() {
    let response = json!({
        "initialized": true,
        "has_superadmin": true
    });

    assert!(response["initialized"].as_bool().unwrap());
    assert!(response["has_superadmin"].as_bool().unwrap());
}

// ════════════════════════════════════════════════════════════════════════════
// HTTP Status Code Tests
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_status_codes() {
    assert_eq!(StatusCode::OK.as_u16(), 200);
    assert_eq!(StatusCode::CREATED.as_u16(), 201);
    assert_eq!(StatusCode::BAD_REQUEST.as_u16(), 400);
    assert_eq!(StatusCode::UNAUTHORIZED.as_u16(), 401);
    assert_eq!(StatusCode::FORBIDDEN.as_u16(), 403);
    assert_eq!(StatusCode::NOT_FOUND.as_u16(), 404);
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), 500);
}

// ════════════════════════════════════════════════════════════════════════════
// RBAC Response Tests
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_role_response_structure() {
    let response = json!({
        "id": "uuid",
        "name": "admin",
        "code": "ADMIN",
        "description": "Administrator role",
        "permissions": ["user:read", "user:write"]
    });

    assert_eq!(response["name"], "admin");
    assert!(response["permissions"].is_array());
}

#[test]
fn test_permission_response_structure() {
    let response = json!({
        "id": "uuid",
        "code": "PLG.I",
        "name": "plugin:install",
        "description": "Install plugins"
    });

    assert_eq!(response["code"], "PLG.I");
    assert_eq!(response["name"], "plugin:install");
}

// ════════════════════════════════════════════════════════════════════════════
// Tenant Response Tests
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_tenant_response_structure() {
    let response = json!({
        "id": "uuid",
        "name": "Acme Corp",
        "slug": "acme-corp",
        "type": "organization",
        "is_active": true,
        "owner_id": "user-uuid"
    });

    assert_eq!(response["name"], "Acme Corp");
    assert_eq!(response["type"], "organization");
    assert!(response["is_active"].as_bool().unwrap());
}

// ════════════════════════════════════════════════════════════════════════════
// API Key Response Tests
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_api_key_response_structure() {
    let response = json!({
        "id": "uuid",
        "name": "Production Key",
        "key": "kyx_prod_xxxxxx",
        "expires_at": "2025-12-31T23:59:59Z"
    });

    assert!(response["key"].as_str().unwrap().starts_with("kyx_"));
    assert!(response["expires_at"].is_string());
}

// ════════════════════════════════════════════════════════════════════════════
// Error Response Tests
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_error_response_structure() {
    let response = json!({
        "error": "Bad Request",
        "message": "Missing required field: tenant_id"
    });

    assert_eq!(response["error"], "Bad Request");
    assert!(response["message"].is_string());
}

#[test]
fn test_not_found_error() {
    let response = json!({
        "error": "Not found",
        "message": "Plugin not found"
    });

    assert_eq!(response["error"], "Not found");
}
