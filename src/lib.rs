use std::path::Path;

use toc::TableOfContent;

pub(crate) mod container;
pub(crate) mod opf;
pub(crate) mod toc;
mod epub;
pub mod error;

/// Epub book
pub trait Epub {
    
    // The book's title
    fn title() -> String;
    
    // The book's cover image
    fn cover() -> Path;

    // The book's table of content
    fn toc() -> TableOfContent;
}