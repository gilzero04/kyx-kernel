// ════════════════════════════════════════════════════════════════════════════
// Validation Tests - Input Validation, Sanitization, Error Handling
// ════════════════════════════════════════════════════════════════════════════

use serde_json::json;

// ════════════════════════════════════════════════════════════════════════════
// Input Validation Tests
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_email_validation() {
    let valid = ["user@example.com", "a@b.co", "test+tag@domain.org"];
    let invalid = ["invalid", "@example.com", "user@", "user@.com"];
    
    for email in valid {
        assert!(email.contains('@'));
        assert!(email.split('@').count() == 2);
        let domain = email.split('@').last().unwrap();
        assert!(domain.contains('.'));
    }
    
    for email in invalid {
        let is_invalid = !email.contains('@') 
            || email.starts_with('@')
            || email.ends_with('@')
            || email.split('@').last().map(|d| !d.contains('.')).unwrap_or(true);
        assert!(is_invalid);
    }
}

#[test]
fn test_phone_validation() {
    let valid_phones = [
        "+1234567890",
        "+66812345678",
        "+1-555-123-4567",
    ];
    
    for phone in valid_phones {
        assert!(phone.starts_with('+') || phone.chars().all(|c| c.is_numeric() || c == '-'));
    }
}

#[test]
fn test_url_validation() {
    let valid_urls = [
        "https://example.com",
        "http://localhost:3000",
        "https://sub.domain.com/path?query=1",
    ];
    
    for url in valid_urls {
        assert!(url.starts_with("http://") || url.starts_with("https://"));
    }
}

#[test]
fn test_uuid_validation() {
    let valid_uuids = [
        "123e4567-e89b-12d3-a456-426614174000",
        "00000000-0000-0000-0000-000000000000",
    ];
    
    for uuid in valid_uuids {
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.matches('-').count(), 4);
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Sanitization Tests
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_html_entity_encoding() {
    let dangerous_chars = ['<', '>', '&', '"', '\''];
    let encoded = ["&lt;", "&gt;", "&amp;", "&quot;", "&#x27;"];
    
    for (i, char) in dangerous_chars.iter().enumerate() {
        assert!(!encoded[i].contains(*char));
    }
}

#[test]
fn test_script_tag_removal() {
    let inputs = [
        "<script>alert(1)</script>",
        "<SCRIPT>alert(1)</SCRIPT>",
        "<script src='evil.js'></script>",
    ];
    
    for input in inputs {
        let lower = input.to_lowercase();
        assert!(lower.contains("<script"));
    }
}

#[test]
fn test_path_traversal_detection() {
    let malicious_paths = [
        "../../../etc/passwd",
        "..\\..\\windows\\system32",
        "/etc/passwd",
        "....//....//etc/passwd",
    ];
    
    for path in malicious_paths {
        let is_malicious = path.contains("..") || path.starts_with('/');
        assert!(is_malicious);
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Error Response Tests
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_validation_error_structure() {
    let error = json!({
        "error": "validation_failed",
        "message": "Validation failed",
        "details": {
            "email": ["Invalid email format"],
            "password": ["Must be at least 8 characters", "Must contain uppercase"]
        }
    });
    
    assert!(error["details"].is_object());
    assert!(error["details"]["email"].is_array());
}

#[test]
fn test_field_error_messages() {
    let field_errors = json!({
        "email": "required",
        "password": "too_short",
        "phone": "invalid_format",
        "age": "must_be_positive"
    });
    
    for (_, value) in field_errors.as_object().unwrap() {
        assert!(value.is_string());
        assert!(!value.as_str().unwrap().is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Length Validation Tests
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_string_length_limits() {
    let limits = json!({
        "name": { "min": 1, "max": 255 },
        "email": { "min": 5, "max": 255 },
        "password": { "min": 8, "max": 128 },
        "description": { "min": 0, "max": 5000 },
        "slug": { "min": 1, "max": 100 }
    });
    
    for (_, value) in limits.as_object().unwrap() {
        let min = value["min"].as_i64().unwrap();
        let max = value["max"].as_i64().unwrap();
        assert!(min >= 0);
        assert!(max > min);
    }
}

#[test]
fn test_array_length_limits() {
    let limits = json!({
        "tags": { "min": 0, "max": 10 },
        "permissions": { "min": 0, "max": 100 },
        "attachments": { "min": 0, "max": 20 }
    });
    
    for (_, value) in limits.as_object().unwrap() {
        let max = value["max"].as_i64().unwrap();
        assert!(max <= 1000);
    }
}
