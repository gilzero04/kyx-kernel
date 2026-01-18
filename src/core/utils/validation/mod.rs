use serde_json::{Value, json};
use validator::ValidationErrors;

#[allow(dead_code)]
pub fn format_validation_errors(errors: ValidationErrors) -> Value {
    let mut details = json!({});

    for (field, field_errors) in errors.field_errors() {
        let field_err_msgs: Vec<String> = field_errors
            .iter()
            .map(|e| {
                e.message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| e.code.to_string())
            })
            .collect();

        details[field] = json!(field_err_msgs);
    }

    details
}
