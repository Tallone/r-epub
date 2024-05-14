use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

pub use chapter::Chapter;
pub use epub::EpubContainer;

pub(crate) mod chapter;
pub(crate) mod container;
mod epub;
pub(crate) mod opf;
pub mod result;
pub(crate) mod toc;
pub(crate) mod types;

/// Epub book
pub trait Epub {
    /// The book's title
    fn title(&self) -> String;

    /// The book's cover image
    fn cover(&self) -> Option<PathBuf>;

    /// The book's table of content
    fn toc(&self) -> &toc::Toc;

    /// All chapters fo the book
    fn chapters(&self) -> Vec<Chapter>;

    /// Get chapter info with given index, start from 1
    fn get_chapter(&self, index: usize) -> Option<Chapter>;
}

/// Useful fn to extract epub to dest_path
pub fn extract_epub<P: AsRef<Path>>(epub_file: P, dest_folder: P) -> result::Result<()> {
    let f = File::open(epub_file)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(f))?;
    if !dest_folder.as_ref().exists() {
        fs::create_dir_all(&dest_folder)?;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_folder.as_ref().join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}
