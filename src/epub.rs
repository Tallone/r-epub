use std::{fs::{self, File}, io::{self, BufReader}, path::{Path, PathBuf}};

use zip::ZipArchive;

use crate::{container::Container, error::Result, opf, toc, Epub};

const ENTRY_FILE: &str = "META-INF/container.xml";

pub struct EpubContainer {
    pub opf: opf::Opf,
    pub toc: toc::Toc,
}

impl EpubContainer {
    /// Parse epub info and if pass extract_path, will extract all files from epub to the path
    pub fn parse<P: AsRef<Path>>(path: P, extract_path: Option<P>) -> Result<EpubContainer> {
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
        let toc: toc::Toc = quick_xml::de::from_reader(BufReader::new(toc_file))?;

        // extract
        if let Some(dest_path) = extract_path {
            Self::extract(&mut archive, dest_path);
        }

        Ok(Self { opf, toc })
    }

    fn extract<P: AsRef<Path>>(archive: &mut ZipArchive<File>, path: P) {
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = path.as_ref().to_path_buf();

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    println!("File {i} comment: {comment}");
                }
            }

            if file.is_dir() {
                fs::create_dir_all(&outpath).unwrap();
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }
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
}

#[cfg(test)]
mod tests {

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
}
