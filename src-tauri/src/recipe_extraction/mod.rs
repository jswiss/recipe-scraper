//! Recipe extraction module for parsing recipe data from HTML content.
//!
//! This module provides functionality to extract structured recipe data from HTML
//! using multiple strategies:
//! 1. JSON-LD structured data (priority 1)
//! 2. Microdata/itemscope markup (priority 2)
//! 3. Local AI fallback for unstructured content (priority 3)

mod commands;
mod duration;
mod json_ld;
mod microdata;
mod models;

pub use commands::extract_recipe;
pub use models::{
    ExtractedField, ExtractedRecipe, ExtractionError, ExtractionSource, Ingredient, Instruction,
    NutritionInfo,
};
