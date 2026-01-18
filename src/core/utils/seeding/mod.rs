use serde_json::{Value, json};
use uuid::Uuid;

pub fn get_default_home_page_content(org_name: &str) -> Value {
    json!({
        "blocks": [
            {
                "id": Uuid::new_v4().to_string(),
                "type": "hero",
                "content": {
                    "title": format!("Welcome to {}", org_name),
                    "subtitle": "Your new multi-tenant application is ready.",
                    "align": "center",
                    "image": "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?q=80&w=2564&auto=format&fit=crop&fm=webp"
                }
            },
            {
                "id": Uuid::new_v4().to_string(),
                "type": "features",
                "content": {
                    "title": "Getting Started",
                    "items": [
                        { "title": "Dashboard", "description": "Manage your users and tenants." },
                        { "title": "CMS", "description": "Create beautiful pages." },
                        { "title": "Settings", "description": "Configure your system." }
                    ]
                }
            }
        ]
    })
}
