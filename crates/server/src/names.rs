//! The one rule for player names, shared by guest nicknames and (later)
//! account usernames.

/// A valid player name: 3-15 ASCII letters or digits. ASCII only, so no
/// lookalike Unicode letters can impersonate another name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerName(String);

impl PlayerName {
    /// Returns `None` unless `raw` follows the rule; surrounding whitespace
    /// is not trimmed, so the client must send the name as shown.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let ok = (3..=15).contains(&raw.len()) && raw.bytes().all(|b| b.is_ascii_alphanumeric());
        ok.then(|| Self(raw.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::PlayerName;

    #[test]
    fn accepts_three_to_fifteen_ascii_letters_or_digits() {
        for ok in ["ana", "Ana2026", "abcdefghijklmno"] {
            assert!(PlayerName::parse(ok).is_some(), "{ok}");
        }
    }

    #[test]
    fn rejects_other_lengths_and_characters() {
        for bad in [
            "",
            "ab",
            "abcdefghijklmnop",
            "ana bob",
            " ana",
            "ana_",
            "anä",
            "аna",
        ] {
            assert!(PlayerName::parse(bad).is_none(), "{bad:?}");
        }
    }
}
