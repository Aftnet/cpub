use chrono::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

const DEFAULT_LANGUAGE: &str = "en-us";

pub struct Metadata {
    id: String,
    title: String,
    author: String,
    publisher: String,
    publishing_date: DateTime<Utc>,
    language: String,
    description: String,
    source: String,
    relation: String,
    copyright: String,
    tags: BTreeSet<String>,
    custom: BTreeMap<String, String>,
    right_to_left: bool,
}

impl Metadata {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_id(&mut self, value: &str) {
        if Metadata::is_invalid_string(value) {
            panic!("Invalid id");
        }
        self.id = value.to_string();
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, value: &str) {
        if Metadata::is_invalid_string(value) {
            panic!("Invalid title");
        }
        self.title = value.to_string();
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn set_author(&mut self, value: &str) {
        if Metadata::is_invalid_string(value) {
            panic!("Invalid author");
        }
        self.author = value.to_string();
    }

    pub fn publisher(&self) -> &str {
        &self.publisher
    }

    pub fn set_publisher(&mut self, value: &str) {
        if Metadata::is_invalid_string(value) {
            panic!("Invalid publisher");
        }
        self.publisher = value.to_string();
    }

    pub fn publishing_date(&self) -> &DateTime<Utc> {
        &self.publishing_date
    }

    pub fn set_publishing_date(&mut self, value: &DateTime<Utc>) {
        self.publishing_date = value.clone();
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn set_language(&mut self, value: &str) {
        if Metadata::is_invalid_string(value) {
            panic!("Invalid language");
        }
        self.language = value.to_string();
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, value: &str) {
        self.description = value.to_string();
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn set_source(&mut self, value: &str) {
        self.description = value.to_string();
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn set_relation(&mut self, value: &str) {
        self.relation = value.to_string();
    }

    pub fn copyright(&self) -> &str {
        &self.copyright
    }

    pub fn set_copyright(&mut self, value: &str) {
        self.copyright = value.to_string();
    }

    pub fn tags(&mut self) -> &mut BTreeSet<String> {
        &mut self.tags
    }

    pub fn custom(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.custom
    }

    pub fn right_to_left(&self) -> bool {
        self.right_to_left
    }

    pub fn set_right_to_left(&mut self, value: bool) {
        self.right_to_left = value;
    }

    fn is_invalid_string(value: &str) -> bool {
        value.is_empty() || value.split_whitespace().count() == 0
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