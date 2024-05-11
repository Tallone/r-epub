
/// The book's table of content(TOC)
#[derive(Default)]
pub struct TableOfContent {
    pub title: String,
    pub depth: u8,
    pub nav_points: Vec<NavPoint>
}

/// Point of navigation
#[derive(Default)]
pub struct NavPoint {
    pub title: String,
    pub page_num: u32,
    pub sub_navpoints: Vec<NavPoint>,
}