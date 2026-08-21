use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{EventId, PublicKey, Topic};

/// A content-addressed image proposed for one ownerless topic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CommunityImage {
    pub sha256: EventId,
    pub url: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub alt: String,
}

impl CommunityImage {
    /// Rejects mutable-looking or unsafe image references at the protocol boundary.
    ///
    /// # Errors
    ///
    /// Returns an explanation for an unsafe URL, unsupported format, invalid dimensions, or alt text.
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.url.starts_with("https://") || self.url.len() > 2_048 {
            return Err("community image URL must be a bounded HTTPS URL");
        }
        if !matches!(
            self.mime_type.as_str(),
            "image/png" | "image/jpeg" | "image/webp"
        ) {
            return Err("community image MIME type is unsupported");
        }
        if self.width == 0 || self.height == 0 || self.width > 4_096 || self.height > 4_096 {
            return Err("community image dimensions are invalid");
        }
        if self.alt.trim().is_empty() || self.alt.len() > 280 {
            return Err("community image alt text is empty or too long");
        }
        Ok(())
    }
}

/// One person's replaceable appearance choice for one bare topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunityAppearance {
    pub author: PublicKey,
    pub topic: Topic,
    pub image: Option<CommunityImage>,
    pub created_at: u64,
    pub event_id: Option<EventId>,
}

impl CommunityAppearance {
    /// Validates the optional image carried by this current choice.
    ///
    /// # Errors
    ///
    /// Returns an explanation when the image metadata is invalid.
    pub fn validate(&self) -> Result<(), &'static str> {
        if let Some(image) = &self.image {
            image.validate()?;
        }
        Ok(())
    }
}

/// Viewer-selected inputs for deterministic appearance convergence.
#[derive(Clone, Copy)]
pub struct AppearanceInput<'a> {
    pub persona: &'a PublicKey,
    pub topic: &'a Topic,
    pub selected_sources: &'a BTreeSet<PublicKey>,
    pub complete_sources: &'a BTreeSet<PublicKey>,
    pub appearances: &'a [CommunityAppearance],
}

/// The effective image and inspectable support behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppearanceResult {
    pub image: Option<CommunityImage>,
    pub direct: bool,
    pub sources: Vec<PublicKey>,
    pub incomplete_sources: Vec<PublicKey>,
}

/// Resolves direct choice first, then distinct support among explicitly selected sources.
pub fn evaluate_appearance(input: AppearanceInput<'_>) -> AppearanceResult {
    let mut current: BTreeMap<&PublicKey, &CommunityAppearance> = BTreeMap::new();
    for choice in input
        .appearances
        .iter()
        .filter(|choice| &choice.topic == input.topic)
    {
        if choice.validate().is_err() {
            continue;
        }
        let replace = current.get(&choice.author).is_none_or(|prior| {
            choice.created_at > prior.created_at
                || (choice.created_at == prior.created_at && choice.event_id < prior.event_id)
        });
        if replace {
            current.insert(&choice.author, choice);
        }
    }
    if let Some(choice) = current.get(input.persona)
        && choice.image.is_some()
    {
        return AppearanceResult {
            image: choice.image.clone(),
            direct: true,
            sources: vec![input.persona.clone()],
            incomplete_sources: Vec::new(),
        };
    }

    let mut support: BTreeMap<EventId, (BTreeSet<PublicKey>, u64, CommunityImage)> =
        BTreeMap::new();
    for source in input.selected_sources {
        let Some(choice) = current.get(source) else {
            continue;
        };
        let Some(image) = &choice.image else { continue };
        let entry = support
            .entry(image.sha256.clone())
            .or_insert_with(|| (BTreeSet::new(), choice.created_at, image.clone()));
        entry.0.insert(source.clone());
        entry.1 = entry.1.max(choice.created_at);
        if image < &entry.2 {
            entry.2 = image.clone();
        }
    }
    let selected = support.into_iter().max_by(|left, right| {
        left.1
            .0
            .len()
            .cmp(&right.1.0.len())
            .then_with(|| left.1.1.cmp(&right.1.1))
            .then_with(|| right.1.2.cmp(&left.1.2))
            .then_with(|| right.0.cmp(&left.0))
    });
    AppearanceResult {
        image: selected.as_ref().map(|(_, (_, _, image))| image.clone()),
        direct: false,
        sources: selected.map_or_else(Vec::new, |(_, (sources, _, _))| {
            sources.into_iter().collect()
        }),
        incomplete_sources: input
            .selected_sources
            .difference(input.complete_sources)
            .cloned()
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(value: char) -> PublicKey {
        PublicKey::parse(value.to_string().repeat(64)).unwrap()
    }
    fn id(value: char) -> EventId {
        EventId::parse(value.to_string().repeat(64)).unwrap()
    }
    fn image(value: char) -> CommunityImage {
        CommunityImage {
            sha256: id(value),
            url: format!("https://images.example/{value}.png"),
            mime_type: "image/png".into(),
            width: 256,
            height: 256,
            alt: "Topic icon".into(),
        }
    }
    fn choice(author: char, image: Option<CommunityImage>, created_at: u64) -> CommunityAppearance {
        CommunityAppearance {
            author: key(author),
            topic: Topic::parse("science").unwrap(),
            image,
            created_at,
            event_id: Some(id(author)),
        }
    }

    #[test]
    fn direct_choice_wins_and_followed_sources_converge_by_distinct_support() {
        let persona = key('a');
        let sources = BTreeSet::from([key('b'), key('c'), key('d')]);
        let complete = sources.clone();
        let shared = image('1');
        let other = image('2');
        let choices = vec![
            choice('b', Some(shared.clone()), 1),
            choice('c', Some(shared.clone()), 2),
            choice('d', Some(other), 9),
        ];
        let result = evaluate_appearance(AppearanceInput {
            persona: &persona,
            topic: &Topic::parse("science").unwrap(),
            selected_sources: &sources,
            complete_sources: &complete,
            appearances: &choices,
        });
        assert_eq!(result.image, Some(shared.clone()));
        assert_eq!(result.sources.len(), 2);
        let direct = [choices, vec![choice('a', Some(image('3')), 3)]].concat();
        let result = evaluate_appearance(AppearanceInput {
            persona: &persona,
            topic: &Topic::parse("science").unwrap(),
            selected_sources: &sources,
            complete_sources: &complete,
            appearances: &direct,
        });
        assert!(result.direct);
        assert_eq!(result.image.unwrap().sha256, id('3'));
    }

    #[test]
    fn equal_hashes_aggregate_despite_metadata_and_direct_withdrawal_reveals_them() {
        let persona = key('a');
        let sources = BTreeSet::from([key('b'), key('c')]);
        let complete = sources.clone();
        let first = image('1');
        let mut alternate = first.clone();
        alternate.url = "https://cdn.example/other.png".into();
        alternate.alt = "Alternate description".into();
        let choices = vec![
            choice('a', None, 10),
            choice('b', Some(first.clone()), 1),
            choice('c', Some(alternate), 2),
        ];
        let result = evaluate_appearance(AppearanceInput {
            persona: &persona,
            topic: &Topic::parse("science").unwrap(),
            selected_sources: &sources,
            complete_sources: &complete,
            appearances: &choices,
        });
        assert!(!result.direct);
        assert_eq!(result.sources.len(), 2);
        assert_eq!(result.image.unwrap().sha256, id('1'));
    }
}
