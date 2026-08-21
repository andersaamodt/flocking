use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{CONFIG_VERSION, Completeness, ContentTarget, Error, Faculty, PublicKey, Scope, Topic};

/// A portable local Flocking configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub persona: PublicKey,
    pub sources: Vec<Source>,
    /// People whose current community-image choices this persona follows.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub appearance_sources: BTreeSet<PublicKey>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_pin_dismissals: Vec<LocalPinDismissal>,
}

impl Config {
    /// Parses and validates a UTF-8 JSON configuration.
    ///
    /// # Errors
    ///
    /// Returns a descriptive error for malformed JSON or an invalid v1 constraint.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let config: Self =
            serde_json::from_str(json).map_err(|error| Error::InvalidConfig(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    /// Validates all cross-record constraints in the portable format.
    ///
    /// # Errors
    ///
    /// Returns an error for unknown versions, duplicates, or invalid grant ranks.
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != CONFIG_VERSION {
            return Err(Error::UnknownConfigVersion);
        }
        let mut source_keys = BTreeSet::new();
        let mut ranks: BTreeMap<Faculty, BTreeSet<u32>> = BTreeMap::new();
        for source in &self.sources {
            if !source_keys.insert(source.pubkey.clone()) {
                return Err(Error::DuplicateSource(source.pubkey.to_string()));
            }
            let mut faculties = BTreeSet::new();
            for grant in &source.grants {
                if !faculties.insert(grant.faculty) {
                    return Err(Error::DuplicateGrant {
                        source_key: source.pubkey.to_string(),
                        faculty: grant.faculty.to_string(),
                    });
                }
                grant.validate()?;
                if let Some(rank) = grant.rank
                    && !ranks.entry(grant.faculty).or_default().insert(rank)
                {
                    return Err(Error::DuplicateRank {
                        faculty: grant.faculty.to_string(),
                        rank,
                    });
                }
            }
        }
        if self.appearance_sources.contains(&self.persona) {
            return Err(Error::InvalidConfig(
                "a persona cannot follow its own appearance choices".to_owned(),
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn source(&self, pubkey: &PublicKey) -> Option<&Source> {
        self.sources.iter().find(|source| &source.pubkey == pubkey)
    }

    #[must_use]
    pub fn grant(&self, pubkey: &PublicKey, faculty: Faculty) -> Option<&FacultyGrant> {
        self.source(pubkey)?
            .grants
            .iter()
            .find(|grant| grant.faculty == faculty)
    }
}

/// One person whose authored judgments may influence the configured persona.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub pubkey: PublicKey,
    pub grants: Vec<FacultyGrant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reverse_blocks: Option<ReverseBlockGrant>,
}

/// The scopes and precedence granted to one source for one faculty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacultyGrant {
    pub faculty: Faculty,
    pub global: bool,
    #[serde(default)]
    pub topics: BTreeSet<Topic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
}

impl FacultyGrant {
    fn validate(&self) -> Result<(), Error> {
        if self.faculty == Faculty::Follow && !self.topics.is_empty() {
            return Err(Error::InvalidGrantScope {
                faculty: self.faculty.to_string(),
                scope: "topic".to_owned(),
            });
        }
        if matches!(self.faculty, Faculty::CommunityMembership | Faculty::Pin) && self.global {
            return Err(Error::InvalidGrantScope {
                faculty: self.faculty.to_string(),
                scope: "global".to_owned(),
            });
        }
        match (self.faculty, self.rank) {
            (Faculty::Pin, Some(_)) => Err(Error::PinHasRank),
            (Faculty::Pin, None) => Ok(()),
            (_, None) => Err(Error::MissingRank),
            (_, Some(0)) => Err(Error::InvalidRank),
            (_, Some(_)) => Ok(()),
        }
    }

    #[must_use]
    pub fn enables(&self, scope: &Scope) -> bool {
        match scope {
            Scope::Global => self.global,
            Scope::Topic(topic) => self.topics.contains(topic),
        }
    }
}

/// Scopes enabled for discovery through another person's blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReverseBlockGrant {
    pub global: bool,
    #[serde(default)]
    pub topics: BTreeSet<Topic>,
}

impl ReverseBlockGrant {
    #[must_use]
    pub fn enables(&self, scope: &Scope) -> bool {
        match scope {
            Scope::Global => self.global,
            Scope::Topic(topic) => self.topics.contains(topic),
        }
    }
}

/// A local-only dismissal of an inherited contextual pin.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LocalPinDismissal {
    pub topic: Topic,
    pub target_type: PinTargetType,
    pub target: String,
}

impl LocalPinDismissal {
    /// Parses the separately tagged target into the core content identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the imported target is malformed or noncanonical.
    pub fn content_target(&self) -> Result<ContentTarget, Error> {
        match self.target_type {
            PinTargetType::Event => crate::EventId::parse(&self.target).map(ContentTarget::Event),
            PinTargetType::Address => ContentTarget::address(&self.target),
        }
    }
}

/// Wire spelling for local pin target identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PinTargetType {
    #[serde(rename = "e")]
    Event,
    #[serde(rename = "a")]
    Address,
}

/// Completeness of one enabled source/faculty/scope input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceState {
    pub source: PublicKey,
    pub faculty: Faculty,
    pub scope: Scope,
    pub completeness: Completeness,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(digit: char) -> PublicKey {
        PublicKey::parse(digit.to_string().repeat(64)).unwrap()
    }

    #[test]
    fn rejects_duplicate_ranks_across_sources() {
        let grant = FacultyGrant {
            faculty: Faculty::Block,
            global: true,
            topics: BTreeSet::new(),
            rank: Some(1),
        };
        let config = Config {
            version: CONFIG_VERSION.to_owned(),
            persona: key('0'),
            sources: vec![
                Source {
                    pubkey: key('1'),
                    grants: vec![grant.clone()],
                    reverse_blocks: None,
                },
                Source {
                    pubkey: key('2'),
                    grants: vec![grant],
                    reverse_blocks: None,
                },
            ],
            appearance_sources: BTreeSet::new(),
            local_pin_dismissals: Vec::new(),
        };
        assert_eq!(
            config.validate(),
            Err(Error::DuplicateRank {
                faculty: "block".to_owned(),
                rank: 1
            })
        );
    }

    #[test]
    fn pin_grants_reject_rank() {
        let grant = FacultyGrant {
            faculty: Faculty::Pin,
            global: false,
            topics: BTreeSet::new(),
            rank: Some(1),
        };
        assert_eq!(grant.validate(), Err(Error::PinHasRank));
    }

    #[test]
    fn grants_reject_faculty_scopes_that_cannot_be_judged() {
        let follow_topic = FacultyGrant {
            faculty: Faculty::Follow,
            global: true,
            topics: BTreeSet::from([Topic::parse("science").unwrap()]),
            rank: Some(1),
        };
        let global_pin = FacultyGrant {
            faculty: Faculty::Pin,
            global: true,
            topics: BTreeSet::new(),
            rank: None,
        };
        assert!(matches!(
            follow_topic.validate(),
            Err(Error::InvalidGrantScope { .. })
        ));
        assert!(matches!(
            global_pin.validate(),
            Err(Error::InvalidGrantScope { .. })
        ));
    }

    #[test]
    fn imported_configuration_revalidates_duplicates() {
        let source = format!(r#"{{"pubkey":"{}","grants":[]}}"#, "1".repeat(64));
        let json = format!(
            r#"{{"version":"flocking-config/1","persona":"{}","sources":[{source},{source}]}}"#,
            "0".repeat(64)
        );
        assert!(matches!(
            Config::from_json(&json),
            Err(Error::DuplicateSource(_))
        ));
    }
}
