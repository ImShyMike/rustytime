use once_cell::sync::OnceCell;

#[inline(always)]
pub fn is_production_env() -> bool {
    if let Ok(env) = std::env::var("ENVIRONMENT")
        && env.eq_ignore_ascii_case("production")
    {
        return true;
    }

    if let Ok(prod) = std::env::var("PRODUCTION") {
        let normalized = prod.trim().to_ascii_lowercase();
        return matches!(normalized.as_str(), "true" | "1" | "yes");
    }

    false
}

#[inline(always)]
pub fn use_cloudflare_headers() -> bool {
    static USE_CLOUDFLARE: OnceCell<bool> = OnceCell::new();
    *USE_CLOUDFLARE.get_or_init(|| {
        std::env::var("USE_CLOUDFLARE_IP_RESOLVER")
            .or_else(|_| std::env::var("USE_CLOUDFLARE"))
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "true" | "1" | "yes" | "on"
                )
            })
            .unwrap_or(false)
    })
}
