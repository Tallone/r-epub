use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, EpubError>;

#[derive(Error, Debug)]
pub enum EpubError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error("Invalid EPUB file: {0}")]
    InvalidEpub(#[from] zip::result::ZipError),

    #[error("Deserialize xml failed: {0}")]
    XmlDeserilize(#[from] quick_xml::DeError),
}
