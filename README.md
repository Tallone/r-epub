
## How to use

``` rust

let path = "demo.epub";
let extract_path = Some("demo");
let epub = EpubContainer::parse(path, extract_path).unwrap();

// Get cover
let cover = epub.cover();

// Get title
let title = epub.title();

// Get toc
let toc = epub.toc();

// Get chapter
let ch = epub.get_chapter(2).unwrap();
println!("ch: {:?}", ch);
```