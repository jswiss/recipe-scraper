use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::recipe_extraction::ExtractedRecipe;

/// Minimum confidence threshold for including a tag in output (FR-006).
pub const MIN_CONFIDENCE_THRESHOLD: f64 = 0.5;

/// A single categorization label with confidence score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    /// The tag label (e.g., "Italian", "breakfast", "vegan")
    pub label: String,
    /// Confidence score between 0.0 and 1.0 inclusive
    pub confidence: f64,
}

impl Tag {
    pub fn new(label: impl Into<String>, confidence: f64) -> Self {
        Self {
            label: label.into(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// One of three fixed categorization domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagDomain {
    Cuisine,
    Course,
    Diet,
}

/// The complete tagging result for a recipe, organized by domain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagSet {
    /// Cuisine tags, ordered by confidence (highest first)
    pub cuisine: Vec<Tag>,
    /// Course tags, ordered by confidence (highest first)
    pub course: Vec<Tag>,
    /// Diet tags, ordered by confidence (highest first)
    pub diet: Vec<Tag>,
}

impl TagSet {
    /// Creates an empty TagSet with no tags in any domain.
    pub fn empty() -> Self {
        Self {
            cuisine: Vec::new(),
            course: Vec::new(),
            diet: Vec::new(),
        }
    }
}

/// Combined result for the `extract_and_tag` command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaggingResult {
    /// The extracted recipe
    pub recipe: ExtractedRecipe,
    /// The assigned tags
    pub tags: TagSet,
}

/// Error type for tagging operations.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum TaggingError {
    /// Recipe has no usable content for tagging
    #[error("No taggable content: {message}")]
    NoContent { message: String },

    /// Extraction failed (only for extract_and_tag command)
    #[error("Extraction failed: {message}")]
    ExtractionFailed { message: String },
}

/// Dietary property flags for ingredient categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DietaryFlag {
    ContainsMeat,
    ContainsPoultry,
    ContainsFish,
    ContainsDairy,
    ContainsEggs,
    ContainsGluten,
    ContainsNuts,
    ContainsSoy,
    HighCarb,
    HighFat,
    ContainsSugar,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_extraction::ExtractionSource;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("Italian", 0.85);
        assert_eq!(tag.label, "Italian");
        assert!((tag.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tag_confidence_clamped() {
        let high = Tag::new("test", 1.5);
        assert!((high.confidence - 1.0).abs() < f64::EPSILON);

        let low = Tag::new("test", -0.2);
        assert!((low.confidence - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tag_set_empty() {
        let tags = TagSet::empty();
        assert!(tags.cuisine.is_empty());
        assert!(tags.course.is_empty());
        assert!(tags.diet.is_empty());
    }

    #[test]
    fn test_serde_round_trip_tag() {
        let tag = Tag::new("Thai", 0.85);
        let json = serde_json::to_string(&tag).unwrap();
        let deserialized: Tag = serde_json::from_str(&json).unwrap();
        assert_eq!(tag, deserialized);
    }

    #[test]
    fn test_serde_round_trip_tag_set() {
        let tags = TagSet {
            cuisine: vec![Tag::new("Thai", 0.85)],
            course: vec![Tag::new("dinner", 0.7)],
            diet: vec![Tag::new("gluten-free", 0.9)],
        };
        let json = serde_json::to_string(&tags).unwrap();
        let deserialized: TagSet = serde_json::from_str(&json).unwrap();
        assert_eq!(tags, deserialized);
    }

    #[test]
    fn test_serde_round_trip_tagging_error() {
        let err = TaggingError::NoContent {
            message: "empty recipe".into(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"error_type\":\"no_content\""));
    }

    #[test]
    fn test_serde_round_trip_tagging_result() {
        let result = TaggingResult {
            recipe: crate::recipe_extraction::ExtractedRecipe::empty(ExtractionSource::JsonLd),
            tags: TagSet::empty(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TaggingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, deserialized);
    }
}
