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
    fn need_render(&self, id: usize) -> bool;
    fn force_render(&self, id: usize);
    fn render(&mut self, id: usize);
    fn resize(&mut self, size: Size<u32>); // doesnt need id, bc all views should be resized
    fn pixel_buffer(&mut self, id: usize) -> Option<(PixelFormat, Vec<u8>)>;

    fn get_cursor(&self, id: usize) -> Interaction;
    // fn get_icon(&self) -> Image<Handle>;
    fn goto_url(&self, id: usize, url: &Url);
    fn goto_html(&self, id: usize, html: &str);
    fn has_loaded(&self, id: usize) -> Option<bool>;

    fn new_view(&mut self, page_type: PageType, size: iced::Size<u32>) -> usize;
    fn remove_view(&mut self, id: usize);
    fn get_views(&self) -> Vec<View>;
    fn get_view(&self, id: usize) -> View;

    fn refresh(&self, id: usize);
    fn go_forward(&self, id: usize);
    fn go_back(&self, id: usize);
    fn focus(&self, id: usize);
    fn unfocus(&self, id: usize);

    fn scroll(&self, id: usize, delta: mouse::ScrollDelta);
    fn handle_keyboard_event(&self, id: usize, event: keyboard::Event);
    fn handle_mouse_event(&mut self, id: usize, point: Point, event: mouse::Event);
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
