pub mod commands;
pub mod course_tagger;
pub mod cuisine_tagger;
pub mod diet_tagger;
pub mod models;
pub mod scoring;
pub mod vocabulary;

pub use commands::{extract_and_tag, tag_recipe, tag_recipe_from_extracted};
pub use models::{Tag, TagSet, TaggingError, TaggingResult};
