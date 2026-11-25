//! Server settings, read from the environment (see `.env.example`).

use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    /// Where players reach the server, e.g. `https://ricochet.example`;
    /// cookies are marked Secure when it is HTTPS.
    pub public_base_url: String,
    /// Guest starts allowed per client IP within `guest_window_secs`.
    pub guest_starts: usize,
    pub guest_window_secs: u64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("{0} is not a valid socket address")]
    BadBind(String),
    #[error("RR_PUBLIC_BASE_URL must start with http:// or https://, got {0}")]
    BadPublicUrl(String),
    #[error("{0} must be a positive whole number, got {1}")]
    BadLimit(&'static str, String),
}

impl Config {
    /// Reads settings through `var` (the environment in production, a map
    /// in tests). Every setting has a development default.
    ///
    /// # Errors
    /// On an invalid bind address or public URL.
    pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let bind_text = var("RR_BIND").unwrap_or_else(|| "127.0.0.1:8080".into());
        let bind = bind_text
            .parse()
            .map_err(|_| ConfigError::BadBind(bind_text))?;
        Ok(Self {
            bind,
            database_url: var("RR_DATABASE_URL").unwrap_or_else(|| "sqlite://ricochet.db".into()),
            public_base_url: public_base_url(var("RR_PUBLIC_BASE_URL"))?,
            guest_starts: limit(&var, "RR_GUEST_STARTS", 5)?,
            guest_window_secs: limit(&var, "RR_GUEST_WINDOW_SECS", 600)?,
        })
    }
}

impl Config {
    /// Whether cookies must be Secure: the public URL is HTTPS.
    #[must_use]
    pub fn secure_cookies(&self) -> bool {
        // Case-insensitive too, in case a Config was built without from_vars.
        self.public_base_url
            .get(..8)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
    }
}

/// A positive limit from `name`, or `default` when unset. Zero is refused:
/// it would lock everyone out rather than disable the limit.
fn limit<T>(
    var: impl Fn(&str) -> Option<String>,
    name: &'static str,
    default: T,
) -> Result<T, ConfigError>
where
    T: std::str::FromStr + PartialEq + From<u8>,
{
    let Some(raw) = var(name) else {
        return Ok(default);
    };
    match raw.trim().parse::<T>() {
        Ok(n) if n != T::from(0) => Ok(n),
        _ => Err(ConfigError::BadLimit(name, raw)),
    }
}

/// Validates the public URL's scheme and lowercases it, so a typo can't
/// silently drop the Secure flag in production.
fn public_base_url(raw: Option<String>) -> Result<String, ConfigError> {
    let raw = raw.unwrap_or_else(|| "http://localhost:8080".into());
    let (scheme, rest) = raw
        .split_once("://")
        .ok_or_else(|| ConfigError::BadPublicUrl(raw.clone()))?;
    let scheme = scheme.to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https") || rest.is_empty() {
        return Err(ConfigError::BadPublicUrl(raw));
    }
    Ok(format!("{scheme}://{rest}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn config(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect();
        Config::from_vars(|key| map.get(key).cloned())
    }

    #[test]
    fn defaults_suit_local_development() {
        let c = config(&[]).unwrap();
        assert_eq!(c.bind, "127.0.0.1:8080".parse().unwrap());
        assert_eq!(c.database_url, "sqlite://ricochet.db");
        assert_eq!(c.public_base_url, "http://localhost:8080");
        assert_eq!((c.guest_starts, c.guest_window_secs), (5, 600));
    }

    #[test]
    fn reads_every_setting() {
        let c = config(&[
            ("RR_BIND", "0.0.0.0:9000"),
            ("RR_DATABASE_URL", "sqlite://other.db"),
            ("RR_PUBLIC_BASE_URL", "https://ricochet.example"),
            ("RR_GUEST_STARTS", "7"),
            ("RR_GUEST_WINDOW_SECS", " 60 "),
        ])
        .unwrap();
        assert_eq!((c.guest_starts, c.guest_window_secs), (7, 60));
        assert_eq!(c.bind, "0.0.0.0:9000".parse().unwrap());
        assert_eq!(c.database_url, "sqlite://other.db");
        assert_eq!(c.public_base_url, "https://ricochet.example");
    }

    #[test]
    fn cookies_are_secure_exactly_for_https_in_any_case() {
        let url = |u: &str| config(&[("RR_PUBLIC_BASE_URL", u)]);
        assert!(url("https://ricochet.example").unwrap().secure_cookies());
        assert!(url("HTTPS://ricochet.example").unwrap().secure_cookies());
        assert!(!url("http://localhost:8080").unwrap().secure_cookies());
        assert!(!config(&[]).unwrap().secure_cookies());
        let unvalidated = Config {
            public_base_url: "HTTPS://x".into(),
            ..config(&[]).unwrap()
        };
        assert!(unvalidated.secure_cookies(), "even without from_vars");
    }

    #[test]
    fn rejects_a_public_url_without_an_http_scheme() {
        for bad in ["https:/x", "ricochet.example", "ftp://x", "https://"] {
            assert_eq!(
                config(&[("RR_PUBLIC_BASE_URL", bad)]),
                Err(ConfigError::BadPublicUrl(bad.into())),
                "{bad}"
            );
        }
    }

    #[test]
    fn rejects_a_bad_bind_address() {
        assert_eq!(
            config(&[("RR_BIND", "nope")]),
            Err(ConfigError::BadBind("nope".into()))
        );
    }

    #[test]
    fn rejects_a_zero_or_malformed_limit() {
        for name in ["RR_GUEST_STARTS", "RR_GUEST_WINDOW_SECS"] {
            for bad in ["0", "-1", "five", ""] {
                assert_eq!(
                    config(&[(name, bad)]),
                    Err(ConfigError::BadLimit(name, bad.into())),
                    "{name}={bad}"
                );
            }
        }
    }
}
