use std::{
    io::{BufReader, Read},
    path::Path,
};

use zip::read::ZipFile;

use crate::{
    container::Container,
    error::Result,
    opf::{self, Opf},
    toc::TableOfContent,
};

const ENTRY_FILE: &str = "META-INF/container.xml";

pub struct EpubContainer {
    pub opf: Opf,
}

impl EpubContainer {
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<EpubContainer> {
        let file = std::fs::File::open(path)?;
        // Unzip epub
        let mut archive = zip::ZipArchive::new(file)?;
        let container_file = archive.by_name(ENTRY_FILE)?;
        let container: Container = quick_xml::de::from_reader(BufReader::new(container_file))?;

        // Parse opf
        let root_path = container.rootfiles.rootfile.full_path;
        let opf_file = archive.by_name(&root_path)?;
        let opf: opf::Opf = quick_xml::de::from_reader(BufReader::new(opf_file))?;

        // Parse toc
        let toc_path = opf.toc_path();
        let toc_file = archive.by_name(&toc_path)?;
        Self::parse_toc(toc_file);

        Ok(Self { opf })
    }

    fn parse_toc(file: ZipFile) {
        let mut reader = quick_xml::Reader::from_reader(BufReader::new(file));
        reader.trim_text(true);
        reader.expand_empty_elements(true);

        let mut toc = TableOfContent::default();

        let mut count = 0;
        // let mut txt = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                // exits the loop when reaching end of file
                Ok(quick_xml::events::Event::Eof) => break,

                Ok(quick_xml::events::Event::Start(e)) => {
                    let bytes = e.name().0;
                    let tag = String::from_utf8(bytes.to_vec()).unwrap();
                    // println!("tag: {tag}");
                    match bytes {
                        b"meta" => {
                            
                            let span = reader.read_to_end_into(e.to_end().name(), &mut buf).unwrap();
                        }
                        _ => (),
                    }
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    // println!("text: {}", e.unescape().unwrap());
                    // txt.push(e.unescape().unwrap().into_owned())
                }

                // There are several other `Event`s we do not consider here
                _ => (),
            }
            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufReader, Cursor, Read};

    use log::info;

    use crate::{container::Container, opf};

    use super::*;

    #[test]
    fn parse_opf() {
        let path = "demo.epub";
        let epub = EpubContainer::parse(path).unwrap();
        println!("opf: {}", epub.opf.metadata.title);
    }
}
