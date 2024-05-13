use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Meta {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@content")]
    pub content: String,
}
