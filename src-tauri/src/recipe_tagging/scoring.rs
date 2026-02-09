use crate::recipe_extraction::ExtractedRecipe;

use super::models::{Tag, MIN_CONFIDENCE_THRESHOLD};

/// Normalizes a raw score to the [0.0, 1.0] range.
pub fn normalize_score(raw: f64, max: f64) -> f64 {
    if max <= 0.0 {
        return 0.0;
    }
    (raw / max).clamp(0.0, 1.0)
}

/// Removes tags below the confidence threshold.
pub fn filter_by_threshold(tags: Vec<Tag>, threshold: f64) -> Vec<Tag> {
    tags.into_iter()
        .filter(|t| t.confidence >= threshold)
        .collect()
}

/// Removes tags below the default minimum confidence threshold (FR-006).
pub fn filter_by_default_threshold(tags: Vec<Tag>) -> Vec<Tag> {
    filter_by_threshold(tags, MIN_CONFIDENCE_THRESHOLD)
}

/// Sorts tags by confidence descending (FR-007).
pub fn sort_by_confidence(tags: &mut [Tag]) {
    tags.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Filters by default threshold and sorts by confidence descending.
pub fn finalize_tags(tags: Vec<Tag>) -> Vec<Tag> {
    let mut tags = filter_by_default_threshold(tags);
    sort_by_confidence(&mut tags);
    tags
}

/// Applies heuristic refinement to cuisine and course tags (FR-016).
///
/// - Co-occurrence bonus: 1.2x multiplier when same tag has signals in title + ingredients
/// - Cross-domain signals: certain cuisine tags boost related course tags
/// - Re-normalizes all scores to [0.0, 1.0] after refinement
pub fn refine_scores(cuisine: &mut Vec<Tag>, course: &mut Vec<Tag>, recipe: &ExtractedRecipe) {
    let title_lower = recipe
        .title
        .value()
        .map(|t| t.to_lowercase())
        .unwrap_or_default();

    let ingredient_text: String = recipe
        .ingredients
        .iter()
        .map(|i| i.name.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    // Co-occurrence bonus for cuisine: if tag label appears in both title and ingredients
    for tag in cuisine.iter_mut() {
        let label_lower = tag.label.to_lowercase();
        let in_title = title_lower.contains(&label_lower);
        let in_ingredients = ingredient_text.contains(&label_lower);
        if in_title && in_ingredients {
            tag.confidence = (tag.confidence * 1.2).min(1.0);
        }
    }

    // Cross-domain: dim sum cuisine boosts appetizer/snack course
    let has_dim_sum = cuisine
        .iter()
        .any(|t| t.label == "Chinese" && title_lower.contains("dim sum"));
    if has_dim_sum {
        for tag in course.iter_mut() {
            if tag.label == "appetizer" || tag.label == "snack" {
                tag.confidence = (tag.confidence * 1.2).min(1.0);
            }
        }
    }

    // Re-normalize: clamp to [0.0, 1.0] and re-filter/sort
    for tag in cuisine.iter_mut().chain(course.iter_mut()) {
        tag.confidence = tag.confidence.clamp(0.0, 1.0);
    }

    cuisine.retain(|t| t.confidence >= MIN_CONFIDENCE_THRESHOLD);
    course.retain(|t| t.confidence >= MIN_CONFIDENCE_THRESHOLD);
    sort_by_confidence(cuisine);
    sort_by_confidence(course);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_extraction::{ExtractedField, ExtractionSource, Ingredient};

    #[test]
    fn test_normalize_score_basic() {
        assert!((normalize_score(1.0, 2.0) - 0.5).abs() < f64::EPSILON);
        assert!((normalize_score(2.0, 2.0) - 1.0).abs() < f64::EPSILON);
        assert!((normalize_score(0.0, 2.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_normalize_score_clamps() {
        assert!((normalize_score(3.0, 2.0) - 1.0).abs() < f64::EPSILON);
        assert!((normalize_score(-1.0, 2.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_normalize_score_zero_max() {
        assert!((normalize_score(1.0, 0.0) - 0.0).abs() < f64::EPSILON);
        assert!((normalize_score(1.0, -1.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_filter_by_threshold() {
        let tags = vec![
            Tag::new("high", 0.8),
            Tag::new("mid", 0.5),
            Tag::new("low", 0.2),
        ];
        let filtered = filter_by_threshold(tags, 0.5);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].label, "high");
        assert_eq!(filtered[1].label, "mid");
    }

    #[test]
    fn test_filter_by_default_threshold() {
        let tags = vec![
            Tag::new("above", 0.6),
            Tag::new("at", 0.5),
            Tag::new("below", 0.4),
        ];
        let filtered = filter_by_default_threshold(tags);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].label, "above");
        assert_eq!(filtered[1].label, "at");
    }

    #[test]
    fn test_sort_by_confidence() {
        let mut tags = vec![
            Tag::new("low", 0.3),
            Tag::new("high", 0.9),
            Tag::new("mid", 0.6),
        ];
        sort_by_confidence(&mut tags);
        assert_eq!(tags[0].label, "high");
        assert_eq!(tags[1].label, "mid");
        assert_eq!(tags[2].label, "low");
    }

    #[test]
    fn test_finalize_tags() {
        let tags = vec![
            Tag::new("low", 0.2),
            Tag::new("mid", 0.6),
            Tag::new("high", 0.9),
            Tag::new("threshold", 0.5),
        ];
        let finalized = finalize_tags(tags);
        assert_eq!(finalized.len(), 3);
        assert_eq!(finalized[0].label, "high");
        assert_eq!(finalized[1].label, "mid");
        assert_eq!(finalized[2].label, "threshold");
    }

    #[test]
    fn test_refine_scores_clamped_to_one() {
        let mut recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        recipe.title = ExtractedField::found("Italian pasta".to_string());
        recipe.ingredients = vec![Ingredient::new(
            "Italian sausage",
            None,
            None,
            "Italian sausage",
        )];

        let mut cuisine = vec![Tag::new("Italian", 0.9)];
        let mut course = vec![Tag::new("dinner", 0.7)];

        refine_scores(&mut cuisine, &mut course, &recipe);

        // Co-occurrence bonus applied but capped at 1.0
        assert!(cuisine[0].confidence <= 1.0);
    }

    #[test]
    fn test_refine_scores_no_crash_on_empty() {
        let recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        let mut cuisine = Vec::new();
        let mut course = Vec::new();
        refine_scores(&mut cuisine, &mut course, &recipe);
        assert!(cuisine.is_empty());
        assert!(course.is_empty());
    }
}
