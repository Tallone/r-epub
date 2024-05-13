use std::{
    fs,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use zip::ZipArchive;

use crate::{chapter::Chapter, container::Container, error::Result, opf, toc, Epub};

const ENTRY_FILE: &str = "META-INF/container.xml";

pub struct EpubContainer {
    folder: Option<PathBuf>,
    opf_path: PathBuf,
    pub opf: opf::Opf,
    pub toc: toc::Toc,
}

impl EpubContainer {
    /// Parse epub info.
    /// If an extract_path is provided, it will extract all files from the EPUB to that path.
    pub fn parse<P: AsRef<Path>>(path: P, extract_path: Option<P>) -> Result<EpubContainer> {
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

        // extract
        if let Some(dest_path) = &extract_path {
            Self::extract(&mut archive, dest_path)?;
        }

        Ok(Self {
            folder: extract_path.map(|f| f.as_ref().to_path_buf()),
            opf_path: Path::new(&opf_path).to_path_buf(),
            opf,
            toc,
        })
    }

    fn extract<P: AsRef<Path>>(archive: &mut ZipArchive<fs::File>, path: P) -> Result<()> {
        let root = path.as_ref().to_path_buf();
        if !root.exists() {
            fs::create_dir_all(&root)?;
        }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => root.join(path),
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
}

impl Epub for EpubContainer {
    fn title(&self) -> String {
        self.opf.metadata.title.clone()
    }

    fn cover(&self) -> Option<String> {
        self.opf.cover_path()
    }

    fn toc(&self) -> &toc::Toc {
        &self.toc
    }

    /// Extract epub before using this method
    fn get_chapter(&self, index: usize) -> Option<Chapter> {
        let items = &self.opf.spine.itemrefs;
        if let Some(folder) = &self.folder {
            if let Some(item) = items.get(index - 1) {
                let id = &item.id_ref;
                if let Some(manifest) = self.opf.get_item(id) {
                    let item_path = get_abs_path(&self.opf_path.to_string_lossy(), &manifest.href);
                    let fs_path = folder.join(item_path);
                    let chapter = Chapter::parse(index, &fs_path.to_string_lossy());
                    return chapter.ok();
                }
            }
        }

        None
    }
}

/// This method will return path relative to epub root path
///
/// Item href in content.opf is relative to the file self, we should convert it to relative to root
fn get_abs_path(opf_path: &str, item_href: &str) -> PathBuf {
    let opf_path = Path::new(opf_path);
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

    use crate::chapter;

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
        let epub = EpubContainer::parse(path, None).unwrap();

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
        let path = "demo.epub";
        let extract_path = Some("target/demo");
        let epub = EpubContainer::parse(path, extract_path).unwrap();
        let ch = epub.get_chapter(2).unwrap();
        println!("ch: {:?}", ch);
    }
}
