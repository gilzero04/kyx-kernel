use rand::seq::SliceRandom;
use base64::{Engine as _, engine::general_purpose};

pub fn generate_default_avatar(full_name: &str) -> String {
    let initials = extract_initials(full_name);
    let gradient = get_random_gradient();
    
    let svg = format!(
        r#"<svg width="100" height="100" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
            <defs>
                <linearGradient id="grad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color:{};stop-opacity:1" />
                    <stop offset="100%" style="stop-color:{};stop-opacity:1" />
                </linearGradient>
            </defs>
            <rect width="100" height="100" fill="url(#grad)" />
            <text x="50%" y="50%" dy=".1em" fill="white" font-family="Arial, sans-serif" font-size="40" font-weight="bold" text-anchor="middle" dominant-baseline="middle">{}</text>
        </svg>"#,
        gradient.0, gradient.1, initials
    );

    let base64_svg = general_purpose::STANDARD.encode(svg);
    format!("data:image/svg+xml;base64,{}", base64_svg)
}

fn extract_initials(full_name: &str) -> String {
    let parts: Vec<&str> = full_name.split_whitespace().collect();
    if parts.is_empty() {
        return "??".to_string();
    }

    if parts.len() == 1 {
        let chars: Vec<char> = parts[0].chars().collect();
        if chars.len() >= 2 {
            return format!("{}{}", chars[0], chars[1]).to_uppercase();
        }
        return format!("{}", chars[0]).to_uppercase();
    }

    let first = parts[0].chars().next().unwrap_or('?');
    let last = parts.last().unwrap_or(&"").chars().next().unwrap_or('?');
    format!("{}{}", first, last).to_uppercase()
}

fn get_random_gradient() -> (&'static str, &'static str) {
    let gradients = vec![
        ("#FF5F6D", "#FFC371"), // Peach
        ("#2193b0", "#6dd5ed"), // Sky
        ("#ee0979", "#ff6a00"), // Sunset
        ("#00b09b", "#96c93d"), // Green
        ("#8E2DE2", "#4A00E0"), // Purple
        ("#4568DC", "#B06AB3"), // Lavender
        ("#f80759", "#bc4e9c"), // Pink
        ("#1f4037", "#99f2c8"), // Emerald
        ("#0575E6", "#021B79"), // Deep Blue
        ("#f12711", "#f5af19"), // Fire
    ];

    let mut rng = rand::thread_rng();
    *gradients.choose(&mut rng).unwrap_or(&("#333333", "#666666"))
}
