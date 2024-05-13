pub(crate) mod container;
pub(crate) mod opf;
pub(crate) mod toc;
pub(crate) mod types;
mod epub;
pub mod error;

/// Epub book
pub trait Epub {
    
    // The book's title
    fn title(&self) -> String;
    
    // The book's cover image
    fn cover(&self) -> Option<String>;

    // The book's table of content
    fn toc(&self) -> &toc::Toc;
}