use chrono::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::errors::MetadataValidationError;

const DEFAULT_LANGUAGE: &str = "en-us";

pub struct Metadata {
    pub id: String,
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub publishing_date: DateTime<Utc>,
    pub language: String,
    pub description: String,
    pub source: String,
    pub relation: String,
    pub copyright: String,
    pub tags: BTreeSet<String>,
    pub custom: BTreeMap<String, String>,
    pub right_to_left: bool,
}

impl Metadata {
    pub fn validate(&self) -> Result<(), MetadataValidationError> {
        fn is_invalid_string(value: &str) -> bool {
            value.is_empty() || value.split_whitespace().count() == 0
        }

        if is_invalid_string(&self.id) {
            return Err(MetadataValidationError { field: "id" });
        }
        if is_invalid_string(&self.title) {
            return Err(MetadataValidationError { field: "title" });
        }
        if is_invalid_string(&self.author) {
            return Err(MetadataValidationError { field: "author" });
        }
        if is_invalid_string(&self.publisher) {
            return Err(MetadataValidationError { field: "publisher" });
        }
        if is_invalid_string(&self.language) {
            return Err(MetadataValidationError { field: "language" });
        }

        return Ok(());
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            id: Uuid::new_v4().to_hyphenated().to_string(),
            title: String::from("Title"),
            author: String::from("Author name"),
            publisher: String::from("Publisher name"),
            publishing_date: Utc::now(),
            language: String::from(DEFAULT_LANGUAGE),
            description: String::default(),
            source: String::default(),
            relation: String::default(),
            copyright: String::default(),
            tags: BTreeSet::default(),
            custom: BTreeMap::default(),
            right_to_left: false,
        }
    }
}