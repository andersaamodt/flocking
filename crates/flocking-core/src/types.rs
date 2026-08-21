use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::Error;

macro_rules! hex32_type {
    ($name:ident, $kind:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Parses a canonical lowercase 32-byte hexadecimal identifier.
            ///
            /// # Errors
            ///
            /// Returns an error unless the input is exactly 64 lowercase hex characters.
            pub fn parse(value: impl Into<String>) -> Result<Self, Error> {
                let value = value.into();
                if value.len() != 64
                    || !value
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                {
                    return Err(Error::InvalidHex32 { kind: $kind });
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl FromStr for $name {
            type Err = Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::parse(value).map_err(de::Error::custom)
            }
        }
    };
}

hex32_type!(PublicKey, "public key");
hex32_type!(EventId, "event ID");

/// A canonical ownerless topic identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Topic(String);

impl Topic {
    pub const MAX_LEN: usize = 64;

    /// Trims and ASCII-lowercases a bare topic before validating it.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty, overlong, or noncanonical topic.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, Error> {
        let normalized = value.as_ref().trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Err(Error::EmptyTopic);
        }
        if normalized.len() > Self::MAX_LEN {
            return Err(Error::TopicTooLong);
        }
        if !normalized
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(Error::InvalidTopic);
        }
        Ok(Self(normalized))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Serialize for Topic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Topic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// The only two scope forms in Flocking v1.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "topic", rename_all = "snake_case")]
pub enum Scope {
    Global,
    Topic(Topic),
}

impl Scope {
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Self::Global => "global".to_owned(),
            Self::Topic(topic) => format!("topic:{topic}"),
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.key().fmt(formatter)
    }
}

/// Stable content identity across revisions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ContentTarget {
    Event(EventId),
    Address(String),
}

impl<'de> Deserialize<'de> for ContentTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "type", content = "value", rename_all = "snake_case")]
        enum WireTarget {
            Event(EventId),
            Address(String),
        }
        match WireTarget::deserialize(deserializer)? {
            WireTarget::Event(id) => Ok(Self::Event(id)),
            WireTarget::Address(coordinate) => Self::address(coordinate).map_err(de::Error::custom),
        }
    }
}

impl ContentTarget {
    /// Parses a Nostr addressable coordinate of `kind:pubkey:d`.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid kind, public key, field count, or NUL byte.
    pub fn address(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        let mut fields = value.splitn(3, ':');
        let kind = fields.next().unwrap_or_default();
        let Some(pubkey) = fields.next() else {
            return Err(Error::InvalidCoordinate);
        };
        let Some(identifier) = fields.next() else {
            return Err(Error::InvalidCoordinate);
        };
        if kind.is_empty()
            || kind.parse::<u16>().is_err()
            || PublicKey::parse(pubkey).is_err()
            || identifier.contains('\0')
        {
            return Err(Error::InvalidCoordinate);
        }
        Ok(Self::Address(value))
    }

    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Self::Event(id) => format!("e:{id}"),
            Self::Address(coordinate) => format!("a:{coordinate}"),
        }
    }
}

/// A person or logical content object.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Target {
    Person(PublicKey),
    Content(ContentTarget),
}

impl Target {
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Self::Person(pubkey) => format!("p:{pubkey}"),
            Self::Content(content) => content.key(),
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.key().fmt(formatter)
    }
}

/// A concrete Flocking judgment faculty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Faculty {
    Follow,
    Block,
    Silence,
    Hide,
    CommunityMembership,
    Pin,
}

impl fmt::Display for Faculty {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Follow => "follow",
            Self::Block => "block",
            Self::Silence => "silence",
            Self::Hide => "hide",
            Self::CommunityMembership => "community_membership",
            Self::Pin => "pin",
        };
        value.fmt(formatter)
    }
}

impl FromStr for Faculty {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "follow" => Ok(Self::Follow),
            "block" => Ok(Self::Block),
            "silence" => Ok(Self::Silence),
            "hide" => Ok(Self::Hide),
            "community_membership" => Ok(Self::CommunityMembership),
            "pin" => Ok(Self::Pin),
            _ => Err(Error::InvalidQuestion {
                faculty: value.to_owned(),
                scope: "unknown".to_owned(),
                target: "unknown".to_owned(),
            }),
        }
    }
}

/// A faculty-specific authored position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Follow,
    Unfollow,
    Block,
    Unblock,
    Silence,
    Unsilence,
    Hide,
    Unhide,
    Remove,
    Restore,
    Pin,
    Withdraw,
}

impl Action {
    #[must_use]
    pub fn is_withdrawn(self) -> bool {
        self == Self::Withdraw
    }

    /// Returns the ordinary positive/negative meaning, or `None` for withdrawal.
    #[must_use]
    pub fn polarity(self) -> Option<bool> {
        match self {
            Self::Follow | Self::Block | Self::Silence | Self::Hide | Self::Remove | Self::Pin => {
                Some(true)
            }
            Self::Unfollow | Self::Unblock | Self::Unsilence | Self::Unhide | Self::Restore => {
                Some(false)
            }
            Self::Withdraw => None,
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Follow => "follow",
            Self::Unfollow => "unfollow",
            Self::Block => "block",
            Self::Unblock => "unblock",
            Self::Silence => "silence",
            Self::Unsilence => "unsilence",
            Self::Hide => "hide",
            Self::Unhide => "unhide",
            Self::Remove => "remove",
            Self::Restore => "restore",
            Self::Pin => "pin",
            Self::Withdraw => "withdraw",
        };
        value.fmt(formatter)
    }
}

impl FromStr for Action {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(value.to_owned())).map_err(|_| {
            Error::InvalidAction {
                faculty: "unknown".to_owned(),
                action: value.to_owned(),
            }
        })
    }
}

/// Relay-data confidence for one enabled source, faculty, and scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Completeness {
    Complete,
    Stale,
    Unknown,
}

/// A source and source event retained by an aggregated derived result.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceProvenance {
    pub source: PublicKey,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<EventId>,
}

/// The topic context in which a question is evaluated.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Context {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<Topic>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_normalization_matches_hydra() {
        assert_eq!(Topic::parse(" Science_2 ").unwrap().as_str(), "science_2");
        assert_eq!(Topic::parse("/h/science"), Err(Error::InvalidTopic));
    }

    #[test]
    fn identifiers_reject_noncanonical_text() {
        assert!(PublicKey::parse("A".repeat(64)).is_err());
        assert!(EventId::parse("0".repeat(63)).is_err());
    }

    #[test]
    fn coordinates_preserve_colons_in_identifier() {
        let coordinate = format!("30023:{}:essay:one", "1".repeat(64));
        assert_eq!(
            ContentTarget::address(&coordinate).unwrap().key(),
            format!("a:{coordinate}")
        );
    }

    #[test]
    fn coordinates_reject_nul_bytes_at_the_boundary() {
        let coordinate = format!("30023:{}:essay\0hidden", "1".repeat(64));
        assert_eq!(
            ContentTarget::address(coordinate),
            Err(Error::InvalidCoordinate)
        );
    }

    #[test]
    fn coordinates_require_the_d_field_separator() {
        let coordinate = format!("30023:{}", "1".repeat(64));
        assert_eq!(
            ContentTarget::address(coordinate),
            Err(Error::InvalidCoordinate)
        );
    }
}
