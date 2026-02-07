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
pub fn sort_by_confidence(tags: &mut Vec<Tag>) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
