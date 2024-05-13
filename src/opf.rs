use serde::{Deserialize, Serialize};

use crate::types;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Opf {
    pub metadata: Metadata,
    pub manifest: Manifest,
    pub spine: Spine,
    pub guide: Guide,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    #[serde(rename = "contributor")]
    pub contributor: Contributor,
    #[serde(rename = "creator")]
    pub creator: Creator,
    #[serde(rename = "date", default)]
    pub date: String,
    #[serde(rename = "title", default)]
    pub title: String,
    #[serde(rename = "language", default)]
    pub language: String,
    #[serde(rename = "publisher", default)]
    pub publisher: String,
    #[serde(rename = "identifier", default)]
    pub identifiers: Vec<Identifier>,
    #[serde(rename = "meta", default)]
    pub metas: Vec<types::Meta>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Contributor {
    #[serde(rename = "@role")]
    pub role: String,
    #[serde(rename = "$text")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Creator {
    #[serde(rename = "@file-as")]
    pub file_as: String,
    #[serde(rename = "@role")]
    pub role: String,
    #[serde(rename = "$text")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Identifier {
    #[serde(rename = "@scheme")]
    pub scheme: String,
    #[serde(rename = "$value")]
    pub value: String,
    #[serde(rename = "@id", default)]
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    #[serde(rename = "item", default)]
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Item {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@href")]
    pub href: String,
    #[serde(rename = "@media-type")]
    pub media_type: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Spine {
    #[serde(rename = "@toc")]
    pub toc: String,
    #[serde(rename = "itemref", default)]
    pub itemrefs: Vec<ItemRef>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemRef {
    #[serde(rename = "@idref")]
    pub id_ref: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Guide {
    #[serde(default)]
    pub reference: Vec<Reference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Reference {
    #[serde(rename = "@type")]
    pub type_: String,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@href")]
    pub href: String,
}

impl Opf {
    pub fn toc_path(&self) -> String {
        let toc_id = &self.spine.toc;
        let toc_metadata = self
            .manifest
            .items
            .iter()
            .find(|i| i.id.eq(toc_id))
            .unwrap();
        toc_metadata.href.clone()
    }

    /// Get the book's cover href.
    pub fn cover_path(&self) -> Option<String> {
        if let Some(first) = self.manifest.items.first() {
            if first.media_type.contains("image") {
                return Some(first.href.clone());
            }
        }

        None
    }

    /// Get item by id
    pub fn get_item(&self, id: &str) -> Option<&Item> {
        if let Some(item) = self.manifest.items.iter().find(|i| i.id == id) {
            return Some(item);
        }

        None
    }
}
