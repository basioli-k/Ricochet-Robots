//! Server settings, read from the environment (see `.env.example`).

use std::net::SocketAddr;

/// Google sign-in credentials; absent when only guests can play.
#[derive(Clone, PartialEq, Eq)]
pub struct GoogleConfig {
    pub client_id: String,
    /// A secret: never logged (see the `Debug` impl).
    pub client_secret: String,
}

impl std::fmt::Debug for GoogleConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleConfig")
            .field("client_id", &self.client_id)
            .field("client_secret", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    /// Where players reach the server, e.g. `https://ricochet.example`;
    /// the Google redirect URL is built from it.
    pub public_base_url: String,
    pub google: Option<GoogleConfig>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("{0} is not a valid socket address")]
    BadBind(String),
    #[error("set both GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET, or neither")]
    PartialGoogle,
}

impl Config {
    /// Reads settings through `var` (the environment in production, a map
    /// in tests). Every setting has a development default except Google,
    /// which is off unless both of its variables are set.
    ///
    /// # Errors
    /// On an invalid address or half-configured Google credentials.
    pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let bind_text = var("RR_BIND").unwrap_or_else(|| "127.0.0.1:8080".into());
        let bind = bind_text
            .parse()
            .map_err(|_| ConfigError::BadBind(bind_text))?;
        let google = match (var("GOOGLE_CLIENT_ID"), var("GOOGLE_CLIENT_SECRET")) {
            (Some(client_id), Some(client_secret)) => Some(GoogleConfig {
                client_id,
                client_secret,
            }),
            (None, None) => None,
            _ => return Err(ConfigError::PartialGoogle),
        };
        Ok(Self {
            bind,
            database_url: var("RR_DATABASE_URL").unwrap_or_else(|| "sqlite://ricochet.db".into()),
            public_base_url: var("RR_PUBLIC_BASE_URL")
                .unwrap_or_else(|| "http://localhost:8080".into()),
            google,
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
        assert_eq!(c.google, None);
    }

    #[test]
    fn google_needs_both_variables() {
        assert_eq!(
            config(&[("GOOGLE_CLIENT_ID", "id")]),
            Err(ConfigError::PartialGoogle)
        );
        let c = config(&[
            ("GOOGLE_CLIENT_ID", "id"),
            ("GOOGLE_CLIENT_SECRET", "s3cret"),
        ])
        .unwrap();
        assert_eq!(c.google.unwrap().client_id, "id");
    }

    #[test]
    fn rejects_a_bad_bind_address() {
        assert_eq!(
            config(&[("RR_BIND", "nope")]),
            Err(ConfigError::BadBind("nope".into()))
        );
    }

    #[test]
    fn debug_output_never_shows_the_secret() {
        let c = config(&[
            ("GOOGLE_CLIENT_ID", "id"),
            ("GOOGLE_CLIENT_SECRET", "s3cret"),
        ])
        .unwrap();
        assert!(!format!("{c:?}").contains("s3cret"));
    }
}
