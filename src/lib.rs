use std::path::PathBuf;

pub use crate::epub::EpubContainer; 

pub(crate) mod chapter;
pub(crate) mod container;
mod epub;
pub mod result;
pub(crate) mod opf;
pub(crate) mod toc;
pub(crate) mod types;

/// Epub book
pub trait Epub {

    // The book's title
    fn title(&self) -> String;

    // The book's cover image
    fn cover(&self) -> Option<PathBuf>;

    // The book's table of content
    fn toc(&self) -> &toc::Toc;

    // Get chapter info with given index
    fn get_chapter(&self, index: usize) -> Option<chapter::Chapter>;
}
