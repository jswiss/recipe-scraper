pub mod course_tagger;
pub mod cuisine_tagger;
pub mod diet_tagger;
pub mod models;
pub mod scoring;
pub mod vocabulary;

pub use models::{Tag, TagSet, TaggingError, TaggingResult};
