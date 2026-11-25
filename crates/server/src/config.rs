//! Server settings, read from the environment (see `.env.example`).

use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    /// Where players reach the server, e.g. `https://ricochet.example`;
    /// cookies are marked Secure when it is HTTPS.
    pub public_base_url: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("{0} is not a valid socket address")]
    BadBind(String),
}

impl Config {
    /// Reads settings through `var` (the environment in production, a map
    /// in tests). Every setting has a development default.
    ///
    /// # Errors
    /// On an invalid bind address.
    pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let bind_text = var("RR_BIND").unwrap_or_else(|| "127.0.0.1:8080".into());
        let bind = bind_text
            .parse()
            .map_err(|_| ConfigError::BadBind(bind_text))?;
        Ok(Self {
            bind,
            database_url: var("RR_DATABASE_URL").unwrap_or_else(|| "sqlite://ricochet.db".into()),
            public_base_url: var("RR_PUBLIC_BASE_URL")
                .unwrap_or_else(|| "http://localhost:8080".into()),
        })
    }
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
    }

    #[test]
    fn reads_every_setting() {
        let c = config(&[
            ("RR_BIND", "0.0.0.0:9000"),
            ("RR_DATABASE_URL", "sqlite://other.db"),
            ("RR_PUBLIC_BASE_URL", "https://ricochet.example"),
        ])
        .unwrap();
        assert_eq!(c.bind, "0.0.0.0:9000".parse().unwrap());
        assert_eq!(c.database_url, "sqlite://other.db");
        assert_eq!(c.public_base_url, "https://ricochet.example");
    }

    #[test]
    fn rejects_a_bad_bind_address() {
        assert_eq!(
            config(&[("RR_BIND", "nope")]),
            Err(ConfigError::BadBind("nope".into()))
        );
    }
}
