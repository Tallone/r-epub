use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Container {
    pub rootfiles: RootFiles,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RootFiles {
    pub rootfile: RootFile,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RootFile {
    #[serde(rename = "@full-path")]
    pub full_path: String,
    #[serde(rename = "@media-type")]
    pub media_type: String,
}
