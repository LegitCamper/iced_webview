use iced::keyboard;
use iced::mouse::{self, Interaction};
use iced::Size;
// use iced::widget::image::{Handle, Image};
use iced::Point;
use slotmap::DefaultKey;
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
    fn need_render(&self, id: DefaultKey) -> bool;
    fn force_render(&self, id: DefaultKey);
    fn render(&mut self, id: DefaultKey);
    fn resize(&mut self, size: Size<u32>); // doesnt need id, bc all views should be resized
    fn pixel_buffer(&mut self, id: DefaultKey) -> Option<(PixelFormat, Vec<u8>)>;

    fn get_cursor(&self, id: DefaultKey) -> Interaction;
    // fn get_icon(&self) -> Image<Handle>;
    fn goto_url(&self, id: DefaultKey, url: &Url);
    fn goto_html(&self, id: DefaultKey, html: &str);
    fn has_loaded(&self, id: DefaultKey) -> Option<bool>;

    fn new_view(&mut self, page_type: PageType, size: iced::Size<u32>) -> DefaultKey;
    fn remove_view(&mut self, id: DefaultKey);
    fn get_views(&self) -> Vec<View>;
    fn get_view(&self, id: DefaultKey) -> View;

    fn refresh(&self, id: DefaultKey);
    fn go_forward(&self, id: DefaultKey);
    fn go_back(&self, id: DefaultKey);
    fn focus(&self, id: DefaultKey);
    fn unfocus(&self, id: DefaultKey);

    fn scroll(&mut self, id: DefaultKey, delta: mouse::ScrollDelta);
    fn handle_keyboard_event(&mut self, id: DefaultKey, event: keyboard::Event);
    fn handle_mouse_event(&mut self, id: DefaultKey, point: Point, event: mouse::Event);
}

/// Generic View used for external widgets
pub struct View {
    pub title: String,
    pub url: String,
}

/// Allows users to create new views with url or custom html
#[derive(Clone)]
pub enum PageType {
    Url(&'static str),
    Html(&'static str),
}
