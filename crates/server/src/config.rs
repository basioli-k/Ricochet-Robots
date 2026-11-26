//! Server settings, read from the environment (see `.env.example`).

use std::net::SocketAddr;
use std::path::PathBuf;

use crate::rooms::manager::RoomLimits;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    /// The built web client to serve (`npm run build` writes `web/dist`).
    pub web_dir: PathBuf,
    /// Where players reach the server, e.g. `https://ricochet.example`;
    /// cookies are marked Secure when it is HTTPS.
    pub public_base_url: String,
    /// Guest starts allowed per client IP within `guest_window_secs`.
    pub guest_starts: usize,
    pub guest_window_secs: u64,
    /// Open-room caps and room size.
    pub rooms: RoomLimits,
    /// Room creations allowed per client within `room_create_window_secs`.
    pub room_creates: usize,
    pub room_create_window_secs: u64,
    /// Chat messages per player per second.
    pub chat_per_sec: usize,
    /// Other game messages (bids, moves, start, concede, and malformed
    /// ones) per player per second.
    pub actions_per_sec: usize,
    /// Open game sockets per player, and per client address.
    pub sockets_per_player: usize,
    pub sockets_per_client: usize,
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
    /// On an invalid bind address, public URL or limit.
    pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let bind_text = var("RR_BIND").unwrap_or_else(|| "127.0.0.1:8080".into());
        let bind = bind_text
            .parse()
            .map_err(|_| ConfigError::BadBind(bind_text))?;
        Ok(Self {
            bind,
            database_url: var("RR_DATABASE_URL").unwrap_or_else(|| "sqlite://ricochet.db".into()),
            web_dir: var("RR_WEB_DIR").map_or_else(|| "web/dist".into(), PathBuf::from),
            public_base_url: public_base_url(var("RR_PUBLIC_BASE_URL"))?,
            guest_starts: limit(&var, "RR_GUEST_STARTS", 5)?,
            guest_window_secs: limit(&var, "RR_GUEST_WINDOW_SECS", 600)?,
            rooms: RoomLimits {
                global: limit(&var, "RR_ROOMS_MAX", 200)?,
                per_owner: limit(&var, "RR_ROOMS_PER_OWNER", 2)?,
                capacity: limit(&var, "RR_ROOM_CAPACITY", 8)?,
            },
            room_creates: limit(&var, "RR_ROOM_CREATES", 5)?,
            room_create_window_secs: limit(&var, "RR_ROOM_CREATE_WINDOW_SECS", 600)?,
            chat_per_sec: limit(&var, "RR_CHAT_PER_SEC", 1)?,
            actions_per_sec: limit(&var, "RR_ACTIONS_PER_SEC", 10)?,
            sockets_per_player: limit(&var, "RR_SOCKETS_PER_PLAYER", 4)?,
            sockets_per_client: limit(&var, "RR_SOCKETS_PER_CLIENT", 32)?,
        })
    }
}

impl Config {
    /// The `Origin` browsers send from the public URL: its scheme and host
    /// with any non-default port, without a path.
    #[must_use]
    pub fn public_origin(&self) -> String {
        let url = &self.public_base_url;
        let host_start = url.find("://").map_or(0, |i| i + 3);
        let end = url[host_start..]
            .find('/')
            .map_or(url.len(), |i| host_start + i);
        let origin = &url[..end];
        let default_port = if self.secure_cookies() { ":443" } else { ":80" };
        origin
            .strip_suffix(default_port)
            .unwrap_or(origin)
            .to_owned()
    }

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
        assert_eq!(c.web_dir, PathBuf::from("web/dist"));
        assert_eq!(c.public_base_url, "http://localhost:8080");
        assert_eq!((c.guest_starts, c.guest_window_secs), (5, 600));
        assert_eq!(c.rooms, RoomLimits::default());
        assert_eq!((c.room_creates, c.room_create_window_secs), (5, 600));
        assert_eq!((c.chat_per_sec, c.actions_per_sec), (1, 10));
        assert_eq!((c.sockets_per_player, c.sockets_per_client), (4, 32));
    }

    #[test]
    fn the_public_origin_drops_any_path() {
        let origin = |u: &str| {
            config(&[("RR_PUBLIC_BASE_URL", u)])
                .unwrap()
                .public_origin()
        };
        assert_eq!(
            origin("https://ricochet.example"),
            "https://ricochet.example"
        );
        assert_eq!(origin("http://localhost:8080/"), "http://localhost:8080");
        assert_eq!(
            origin("HTTPS://x.example:8443/game/"),
            "https://x.example:8443"
        );
        assert_eq!(origin("https://x.example:443"), "https://x.example");
        assert_eq!(origin("http://x.example:80/"), "http://x.example");
        assert_eq!(origin("http://x.example:443"), "http://x.example:443");
    }

    #[test]
    fn reads_every_setting() {
        let c = config(&[
            ("RR_BIND", "0.0.0.0:9000"),
            ("RR_DATABASE_URL", "sqlite://other.db"),
            ("RR_WEB_DIR", "/srv/web"),
            ("RR_PUBLIC_BASE_URL", "https://ricochet.example"),
            ("RR_GUEST_STARTS", "7"),
            ("RR_GUEST_WINDOW_SECS", " 60 "),
            ("RR_ROOMS_MAX", "50"),
            ("RR_ROOMS_PER_OWNER", "1"),
            ("RR_ROOM_CAPACITY", "4"),
            ("RR_ROOM_CREATES", "3"),
            ("RR_ROOM_CREATE_WINDOW_SECS", "120"),
            ("RR_CHAT_PER_SEC", "2"),
            ("RR_ACTIONS_PER_SEC", "20"),
            ("RR_SOCKETS_PER_PLAYER", "2"),
            ("RR_SOCKETS_PER_CLIENT", "8"),
        ])
        .unwrap();
        assert_eq!(
            c.rooms,
            RoomLimits {
                global: 50,
                per_owner: 1,
                capacity: 4
            }
        );
        assert_eq!((c.room_creates, c.room_create_window_secs), (3, 120));
        assert_eq!((c.chat_per_sec, c.actions_per_sec), (2, 20));
        assert_eq!((c.sockets_per_player, c.sockets_per_client), (2, 8));
        assert_eq!((c.guest_starts, c.guest_window_secs), (7, 60));
        assert_eq!(c.bind, "0.0.0.0:9000".parse().unwrap());
        assert_eq!(c.database_url, "sqlite://other.db");
        assert_eq!(c.web_dir, PathBuf::from("/srv/web"));
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
        for name in [
            "RR_GUEST_STARTS",
            "RR_GUEST_WINDOW_SECS",
            "RR_ROOMS_MAX",
            "RR_ROOMS_PER_OWNER",
            "RR_ROOM_CAPACITY",
            "RR_ROOM_CREATES",
            "RR_ROOM_CREATE_WINDOW_SECS",
            "RR_CHAT_PER_SEC",
            "RR_ACTIONS_PER_SEC",
            "RR_SOCKETS_PER_PLAYER",
            "RR_SOCKETS_PER_CLIENT",
        ] {
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
