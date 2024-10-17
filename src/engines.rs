use iced::keyboard;
use iced::mouse::{self, Interaction};
use iced::Size;
// use iced::widget::image::{Handle, Image};
use iced::Point;
use url::Url;

#[cfg(all(feature = "ultralight", feature = "gosub"))]
compile_error!("Ultralight and Gosub are mutually exclusive and cannot be enabled together");

#[cfg(feature = "ultralight")]
pub mod ultralight;

#[cfg(feature = "gosub")]
pub mod gosub;

pub enum PixelFormat {
    Rgba,
    Bgra,
}

pub trait Engine {
    fn do_work(&self);
    fn need_render(&self) -> bool;
    fn force_render(&self);
    fn render(&mut self);
    fn size(&self) -> Option<(u32, u32)>;
    fn resize(&mut self, size: Size<u32>);
    fn pixel_buffer(&mut self) -> Option<(PixelFormat, Vec<u8>)>;

    fn get_cursor(&self) -> Interaction;
    // fn get_icon(&self) -> Image<Handle>;
    fn goto_url(&self, url: &Url);
    fn goto_html(&self, html: &str);
    fn has_loaded(&self) -> Option<bool>;

    fn refresh(&self);
    fn go_forward(&self);
    fn go_back(&self);
    fn focus(&self);
    fn unfocus(&self);

    fn scroll(&self, delta: mouse::ScrollDelta);
    fn handle_keyboard_event(&self, event: keyboard::Event);
    fn handle_mouse_event(&mut self, point: Point, event: mouse::Event);
}

/// Generic View used for external widgets
pub struct View {
    title: String,
    url: String,
}

/// Allows users to create new views with url or custom html
#[derive(Clone)]
pub enum PageType {
    Url(&'static str),
    Html(&'static str),
}
