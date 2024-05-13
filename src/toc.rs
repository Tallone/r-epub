use serde::{Deserialize, Serialize};

use crate::types;

#[derive(Debug, Serialize, Deserialize)]
pub struct Toc {
    pub head: Head,
    #[serde(rename = "navMap")]
    pub nav_map: NavMap,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Head {
    #[serde(rename = "meta")]
    pub metas: Vec<types::Meta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NavMap {
    #[serde(rename = "navPoint", default)]
    pub nav_points: Vec<NavPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NavPoint {
    #[serde(rename = "@playOrder")]
    pub play_order: u32,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@class")]
    pub class: String,
    #[serde(rename = "navLabel")]
    pub nav_lable: NavLabel,
    #[serde(rename = "navPoint", default)]
    pub sub_navpoints: Vec<NavPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NavLabel {
    pub text: Text,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Text {
    #[serde(rename = "$text")]
    pub text: String,
}

impl Toc {
    pub fn depth(&self) -> usize {
        if let Some(meta) = self.head.metas.iter().find(|m| m.name == "dtb:depth") {
            return meta.content.parse::<usize>().unwrap();
        }
        0
    }
}
