use base64::{Engine as _, engine::general_purpose};
use rand::seq::SliceRandom;

/// Avatar and Cover gradient pairs - each pair has different but complementary colors
/// Index 0 = for avatar, Index 1 = for cover, Index 2-3 = for images
const GRADIENT_SETS: [([&str; 2], [&str; 2], [&str; 2], [&str; 2]); 10] = [
    // (Avatar, Cover, Image1, Image2)
    (
        ["#FF5F6D", "#FFC371"],
        ["#667eea", "#764ba2"],
        ["#11998e", "#38ef7d"],
        ["#FC466B", "#3F5EFB"],
    ), // Peach + Purple
    (
        ["#2193b0", "#6dd5ed"],
        ["#f12711", "#f5af19"],
        ["#8E2DE2", "#4A00E0"],
        ["#00c6fb", "#005bea"],
    ), // Sky + Fire
    (
        ["#ee0979", "#ff6a00"],
        ["#00b09b", "#96c93d"],
        ["#0575E6", "#021B79"],
        ["#f953c6", "#b91d73"],
    ), // Sunset + Green
    (
        ["#00b09b", "#96c93d"],
        ["#f80759", "#bc4e9c"],
        ["#4568DC", "#B06AB3"],
        ["#43e97b", "#38f9d7"],
    ), // Green + Pink
    (
        ["#8E2DE2", "#4A00E0"],
        ["#F7971E", "#FFD200"],
        ["#1f4037", "#99f2c8"],
        ["#a8c0ff", "#3f2b96"],
    ), // Purple + Gold
    (
        ["#4568DC", "#B06AB3"],
        ["#11998e", "#38ef7d"],
        ["#FF5F6D", "#FFC371"],
        ["#6a11cb", "#2575fc"],
    ), // Lavender + Emerald
    (
        ["#f80759", "#bc4e9c"],
        ["#00c6fb", "#005bea"],
        ["#ee0979", "#ff6a00"],
        ["#ff0844", "#ffb199"],
    ), // Pink + Ocean
    (
        ["#1f4037", "#99f2c8"],
        ["#FC466B", "#3F5EFB"],
        ["#2193b0", "#6dd5ed"],
        ["#0cebeb", "#20e3b2"],
    ), // Emerald + Sunset
    (
        ["#0575E6", "#021B79"],
        ["#f953c6", "#b91d73"],
        ["#F7971E", "#FFD200"],
        ["#4481eb", "#04befe"],
    ), // Deep Blue + Magenta
    (
        ["#f12711", "#f5af19"],
        ["#a8c0ff", "#3f2b96"],
        ["#00b09b", "#96c93d"],
        ["#fa709a", "#fee140"],
    ), // Fire + Cool Purple
];

/// Get a random gradient set index
fn get_random_gradient_index() -> usize {
    let mut rng = rand::thread_rng();
    let indices: Vec<usize> = (0..GRADIENT_SETS.len()).collect();
    *indices.choose(&mut rng).unwrap_or(&0)
}

pub fn generate_default_avatar(full_name: &str) -> String {
    let initials = extract_initials(full_name);
    let idx = get_random_gradient_index();
    let gradient = GRADIENT_SETS[idx].0;

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
        gradient[0], gradient[1], initials
    );

    let base64_svg = general_purpose::STANDARD.encode(svg);
    format!("data:image/svg+xml;base64,{}", base64_svg)
}

/// Generate avatar, cover, and images as a set with complementary gradients
pub fn generate_user_images(full_name: &str) -> UserImages {
    let initials = extract_initials(full_name);
    let idx = get_random_gradient_index();
    let gradients = GRADIENT_SETS[idx];

    UserImages {
        avatar_url: generate_gradient_svg(&initials, gradients.0, true),
        cover_url: generate_gradient_svg(&initials, gradients.1, false),
        images: vec![
            generate_gradient_svg(&initials, gradients.2, false),
            generate_gradient_svg(&initials, gradients.3, false),
        ],
    }
}

#[derive(Debug, Clone)]
pub struct UserImages {
    pub avatar_url: String,
    pub cover_url: String,
    pub images: Vec<String>,
}

fn generate_gradient_svg(initials: &str, gradient: [&str; 2], show_initials: bool) -> String {
    let text_element = if show_initials {
        format!(
            r#"<text x="50%" y="50%" dy=".1em" fill="white" font-family="Arial, sans-serif" font-size="40" font-weight="bold" text-anchor="middle" dominant-baseline="middle">{}</text>"#,
            initials
        )
    } else {
        String::new()
    };

    let svg = format!(
        r#"<svg width="400" height="200" viewBox="0 0 400 200" xmlns="http://www.w3.org/2000/svg">
            <defs>
                <linearGradient id="grad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color:{};stop-opacity:1" />
                    <stop offset="100%" style="stop-color:{};stop-opacity:1" />
                </linearGradient>
            </defs>
            <rect width="400" height="200" fill="url(#grad)" />
            {}
        </svg>"#,
        gradient[0], gradient[1], text_element
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_user_images() {
        let images = generate_user_images("Admin User");
        assert!(!images.avatar_url.is_empty());
        assert!(!images.cover_url.is_empty());
        assert_eq!(images.images.len(), 2);
        // Ensure avatar and cover have different base64 content
        assert_ne!(images.avatar_url, images.cover_url);
    }
}
