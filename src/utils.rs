pub struct WallpaperOptions {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}
impl VerticalAlign {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Top,
            1 => Self::Middle,
            _ => Self::Bottom,
        }
    }
}
