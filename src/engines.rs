use iced::keyboard;
use iced::mouse::{self, Interaction};
use iced::Size;
// use iced::widget::image::{Handle, Image};
use iced::Point;

use crate::ImageInfo;

#[cfg(feature = "ultralight")]
pub mod ultralight;

pub enum PixelFormat {
    Rgba,
    Bgra,
}

pub type ViewId = usize;

/// Trait to handle multiple browser engines
/// Currently only supports cpu renders via pixel_buffer
pub trait Engine {
    /// Used to do work in the actual browser engine
    fn update(&mut self);
    /// Has Ultralight perform a new render
    fn render(&mut self, size: Size<u32>);
    /// Request that the browser engine rerender a specific view that may have been updated
    fn request_render(&mut self, id: ViewId);
    /// Creates new a new blank view and returns the ViewId to interact with it
    fn new_view(&mut self, size: Size<u32>) -> ViewId;

    // window changes - no id needed they work for all views
    fn focus(&mut self);
    fn unfocus(&self);
    fn resize(&mut self, size: Size<u32>);

    // handle events per engine
    fn handle_keyboard_event(&mut self, id: ViewId, event: keyboard::Event);
    fn handle_mouse_event(&mut self, id: ViewId, point: Point, event: mouse::Event);

    /// Allows navigating to html or Url on a specific view
    fn goto(&mut self, id: ViewId, page_type: PageType);
    fn refresh(&mut self, id: ViewId);
    fn go_forward(&mut self, id: ViewId);
    fn go_back(&mut self, id: ViewId);
    fn scroll(&mut self, id: ViewId, delta: mouse::ScrollDelta);

    fn get_url(&self, id: ViewId) -> Option<String>;
    fn get_title(&self, id: ViewId) -> Option<String>;
    fn get_view(&self, id: ViewId) -> &ImageInfo;
    fn get_cursor(&self, id: ViewId) -> Interaction;
}

/// Allows users to create new views with url or custom html
#[derive(Clone)]
pub enum PageType {
    Url(&'static str),
    Html(&'static str),
}
