pub use html::root::Html;
use iced::widget::image;

pub mod engines;
pub use engines::{DisplayTab, Engine, PageType, PixelFormat, Tab, TabInfo, Tabs};

pub mod webview;
pub use webview::WebView;

#[cfg(feature = "ultralight")]
pub use engines::ultralight::Ultralight;

/// Image details for passing the view around
#[derive(Clone, Debug, PartialEq)]
pub struct ImageInfo {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            pixels: vec![255; (Self::WIDTH as usize * Self::HEIGHT as usize) * 4],
            width: Self::WIDTH,
            height: Self::HEIGHT,
        }
    }
}

impl ImageInfo {
    // The default dimentions
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 800;

    pub fn new(pixels: Vec<u8>, format: PixelFormat, width: u32, height: u32) -> Self {
        // R, G, B, A
        assert_eq!(pixels.len() % 4, 0);

        let pixels = match format {
            PixelFormat::Rgba => pixels,
            PixelFormat::Bgra => pixels
                .chunks(4)
                .flat_map(|chunk| [chunk[2], chunk[1], chunk[0], chunk[3]])
                .collect(),
        };

        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn as_image(&self) -> image::Image<image::Handle> {
        image::Image::new(image::Handle::from_rgba(
            self.width,
            self.height,
            self.pixels.clone(),
        ))
    }
}

/// Allows different widgets to interact in their native way
#[derive(Debug, Clone, PartialEq)]
pub enum TabSelectionType {
    Id(u32),
    Index(usize),
}
impl Default for TabSelectionType {
    fn default() -> Self {
        TabSelectionType::Index(0)
    }
}
