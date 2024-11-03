//! A library to embed Web views in iced applications. It is a renderer agnostic webview library for Iced.
//!
//! > Note: Currently this library only supports [Ultralight]/Webkit, but more rendering engines are planned to be supported.
//! > [Ultralight has its own licence](https://ultralig.ht/pricing/) that should be reviewed before deciding if it works for you
//!
//! Has two separate widgets: Basic, and Advanced.
//! The basic widget is very simple to implement and requires no knoledge of the widget.
//! You can use simple abstractions like CloseCurrent, and ChangeView. o
//! Whereis with the Advanced widget, you have callbacks when a view is done being created and you need to keep track of the ViewId for view calls
//!
//! #Basic usage should look familiar to iced users:
//! You'll need to create a `Message` for Webview:
//!
//! ```rust
//! enum Message {
//!    WebView(Action),
//!    Update
//! }
//! ```
//!
//! Then you should be able to call the usual view/update methodes:
//!
//! ```rust
//! fn update(state: &mut State, message: Message) {
//!     match message {
//!         Message::WebView(msg) => self.webview.update(msg),
//!         Message::Update => self.webview.update(Action::Update),
//!     }
//! }
//! ```
//!
//! ```rust
//! fn view(state: &mut State, message: Message) -> Element<Message> {
//!    self.webview.view().map(Message::WebView).into()
//! }
//! ```
//!
//! The subscription provides periodic updates so that all the backend rendering is done frequently enough
//!
//! ```rust
//! fn subscription(&self) -> Subscription<Message> {
//!     time::every(Duration::from_millis(10))
//!         .map(|_| Action::Update)
//!         .map(Message::WebView)
//! }
//! ```
//!
//!
//! Examples can be found in the [iced_webview repo](https://github.com/LegitCamper/iced_webview)
//!
use iced::widget::image;

/// Engine Trait and Engine implementations
pub mod engines;
pub use engines::{Engine, PageType, PixelFormat, ViewId};

mod webview;
pub use basic::{Action, WebView};
pub use webview::{advanced, basic}; // pub these since its the default/reccommended method

#[cfg(feature = "ultralight")]
pub use engines::ultralight::Ultralight;

/// Image details for passing the view around
#[derive(Clone, Debug, PartialEq)]
pub struct ImageInfo {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
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

    fn new(pixels: Vec<u8>, format: PixelFormat, width: u32, height: u32) -> Self {
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

    fn as_image(&self) -> image::Image<image::Handle> {
        image::Image::new(image::Handle::from_rgba(
            self.width,
            self.height,
            self.pixels.clone(),
        ))
    }

    fn blank(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![255; (width as usize * height as usize) * 4],
            width,
            height,
        }
    }
}
