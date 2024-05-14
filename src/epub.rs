use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::{
    chapter::Chapter,
    container::Container,
    opf,
    result::{EpubError, Result},
    toc, Epub,
};

const ENTRY_FILE: &str = "META-INF/container.xml";

pub struct EpubContainer {
    folder_path: Option<PathBuf>,
    opf_path: PathBuf,
    opf: opf::Opf,
    toc: toc::Toc,
}

impl EpubContainer {
    /// Parse from epub file.
    pub fn parse_epub<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = fs::File::open(path)?;
        // Unzip epub
        let mut archive = zip::ZipArchive::new(file)?;
        let container_file = archive.by_name(ENTRY_FILE)?;
        let container: Container = quick_xml::de::from_reader(BufReader::new(container_file))?;

        // Parse opf
        let opf_path = container.rootfiles.rootfile.full_path;
        let opf_file = archive.by_name(&opf_path)?;
        let opf: opf::Opf = quick_xml::de::from_reader(BufReader::new(opf_file))?;

        // Parse toc
        let toc_path = opf.toc_path();
        let toc_file = archive.by_name(&toc_path)?;
        let toc: toc::Toc = quick_xml::de::from_reader(BufReader::new(toc_file))?;

        Ok(Self {
            folder_path: None,
            opf_path: Path::new(&opf_path).to_path_buf(),
            opf,
            toc,
        })
    }

    /// Parsing the META-INF/container.xml File from already unzipped EPUB
    pub fn parse_container<P: AsRef<Path>>(container_path: P) -> Result<Self> {
        let ct_path = container_path.as_ref();
        if ct_path.is_dir() || !ct_path.exists() || ct_path.is_relative() {
            return Err(EpubError::Other(
                "Container path should be an absoluted .xml file path".to_owned(),
            ));
        }

        let root_path = ct_path.parent().and_then(|p| p.parent());
        if root_path.is_none() {
            return Err(EpubError::Other(
                "Container path should be in an META-INFO folder".to_owned(),
            ));
        }
        let root_path = root_path.unwrap();

        let container_file = File::open(&container_path)?;
        let container: Container = quick_xml::de::from_reader(BufReader::new(container_file))?;

        // Parse opf
        let opf_path = container.rootfiles.rootfile.full_path;
        let opf_file = File::open(root_path.join(&opf_path))?;
        let opf: opf::Opf = quick_xml::de::from_reader(BufReader::new(opf_file))?;

        // Parse toc
        let toc_path = get_abs_path(&opf_path, &opf.toc_path());
        let toc_file = File::open(root_path.join(toc_path))?;
        let toc: toc::Toc = quick_xml::de::from_reader(BufReader::new(toc_file))?;

        Ok(Self {
            folder_path: Some(root_path.to_path_buf()),
            opf_path: Path::new(&opf_path).to_path_buf(),
            opf,
            toc,
        })
    }
}

impl Epub for EpubContainer {
    fn title(&self) -> String {
        self.opf.metadata.title.clone()
    }

    fn cover(&self) -> Option<PathBuf> {
        self.opf
            .cover_path()
            .map(|p| get_abs_path(&self.opf_path, &Path::new(&p).to_path_buf()))
    }

    fn toc(&self) -> &toc::Toc {
        &self.toc
    }

    fn get_chapter(&self, index: usize) -> Option<Chapter> {
        let items = &self.opf.spine.itemrefs;
        if let Some(folder) = &self.folder_path {
            if let Some(item) = items.get(index - 1) {
                let id = &item.id_ref;
                if let Some(manifest) = self.opf.get_item(id) {
                    let item_path =
                        get_abs_path(&self.opf_path, &Path::new(&manifest.href).to_path_buf());
                    let chapter = Chapter::parse(
                        index,
                        &folder.to_string_lossy(),
                        &item_path.to_string_lossy(),
                    );
                    return chapter.ok();
                }
            }
        }

        None
    }

    fn chapters(&self) -> Vec<Chapter> {
        let mut ret = Vec::new();
        for i in 0..self.opf.spine.itemrefs.len() {
            if let Some(ch) = self.get_chapter(i + 1) {
                ret.push(ch);
            }
        }
        ret
    }
}

/// This method will return path relative to epub root path
///
/// Item href in content.opf is relative to the file self, we should convert it to relative to root
fn get_abs_path<P: AsRef<Path>>(opf_path: P, item_href: P) -> PathBuf {
    let opf_path = opf_path.as_ref();
    let item_href = item_href.as_ref();
    let path_buf = if let Some(parent) = opf_path.parent() {
        parent.join(item_href)
    } else {
        PathBuf::from(item_href)
    };

    // Manually resolve ".." and "." segments in the path without canonicalizing
    let mut components = Vec::new();
    for component in path_buf.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {} // Ignore "." segments
            _ => components.push(component),
        }
    }

    // Reconstruct the path from the components
    let resolved_path = components
        .iter()
        .fold(PathBuf::new(), |mut acc, &component| {
            acc.push(component.as_os_str());
            acc
        });

    resolved_path
}

#[cfg(test)]
mod tests {

    use std::env;

    use self::toc::{NavMap, NavPoint};

    use super::*;

    fn print_nav_map(nav_map: &NavMap, depth: usize) {
        let indent = " ".repeat(depth * 4);
        for nav_point in &nav_map.nav_points {
            print_nav_point(nav_point, &indent);
        }
    }

    fn print_nav_point(nav_point: &NavPoint, indent: &str) {
        println!("{}NavPoint {{", indent);
        println!(
            "{}    nav_lable: \"{}\",",
            indent, nav_point.nav_lable.text.text
        ); // assuming NavLabel has its own Debug implementation
        if !nav_point.sub_navpoints.is_empty() {
            println!("{}    sub_navpoints: [", indent);
            for sub_navpoint in &nav_point.sub_navpoints {
                print_nav_point(sub_navpoint, &format!("{}    ", indent));
            }
            println!("{}    ],", indent);
        }
        println!("{}}},", indent);
    }

    #[test]
    fn parse() {
        let path = "demo.epub";
        let epub = EpubContainer::parse_epub(path).unwrap();

        print_nav_map(&epub.toc.nav_map, epub.toc.depth());
    }

    #[test]
    fn relative_path() {
        let opf = "content.opf";
        let href = "toc.ncx";
        let p = get_abs_path(opf, href);
        assert_eq!(p.to_str().unwrap(), "toc.ncx");

        let href = "1/OEBPS/chapter1.xhtml";
        let p = get_abs_path(opf, href);
        assert_eq!(p.to_str().unwrap(), "1/OEBPS/chapter1.xhtml");

        let opf = "OEBPS/content.opf";
        let href = "toc.ncx";
        let p = get_abs_path(opf, href);
        assert_eq!(p.to_str().unwrap(), "OEBPS/toc.ncx");

        let opf = "OEBPS/content.opf";
        let href = "../toc.ncx";
        let p = get_abs_path(opf, href);
        assert_eq!(p.to_str().unwrap(), "toc.ncx");
    }

    #[test]
    fn get_chapter() {
        let path = "target/demo/META-INF/container.xml";
        let cur_path = env::current_dir().unwrap();

        let epub = EpubContainer::parse_container(cur_path.join(path)).unwrap();
        let ch = epub.get_chapter(2).unwrap();
        println!("ch: {:?}", ch);
    }
}
