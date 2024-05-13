use std::{fs, path::{Path, PathBuf}};

use crate::result::Result;
use scraper::{Html, Selector};

#[derive(Debug)]
pub struct Chapter {
    pub index: usize,
    pub title: String,
    pub path: PathBuf,
    pub word_count: usize,
}

impl Chapter {
    pub fn parse(index: usize, root_path: &str, href: &str) -> Result<Self> {
        let path = Path::new(root_path).join(href);
        let content = fs::read_to_string(path)?;
        let doc = Html::parse_document(&content);

        // parse title
        let mut title = String::default();
        if let Ok(title_sel) = Selector::parse("title") {
            if let Some(ele) = doc.select(&title_sel).next() {
                if let Some(text) = ele.text().next() {
                    title.push_str(text);
                }
            }
        }

        // count word
        let mut word_count = 0;
        if let Ok(body_sel) = Selector::parse("body") {
            if let Some(ele) = doc.select(&body_sel).next() {
                word_count = ele
                    .text()
                    .map(|c| {
                        let s = c.trim();
                        s.chars().count()
                    })
                    .reduce(|acc, e| acc + e)
                    .unwrap_or_default();
            }
        }

        Ok(Self {
            index,
            title,
            word_count,
            path: href.into(),
        })
    }
}
