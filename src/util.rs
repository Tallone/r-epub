use std::path::Path;

use crate::error::Result;

pub struct EpubContainer {
    archive: zip::ZipArchive<std::fs::File>,
}

impl EpubContainer {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<EpubContainer> {
        let file = std::fs::File::open(path)?;
        let archive = zip::ZipArchive::new(file)?;
        Ok(EpubContainer { archive })
    }

    pub fn parse(&mut self) {
        self.archive.file_names().for_each(|name| {
            println!("{}", name);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let path = "C:\\Users\\Tallone\\Desktop\\demo .epub";
        let mut epub = EpubContainer::new(path).unwrap();
        epub.parse();
    }
}
