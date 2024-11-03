use crate::ImageInfo;
use iced::keyboard;
use iced::mouse::{self, Interaction};
use iced::Point;
use iced::Size;

/// A Ultralight implementation of Engine
#[cfg(feature = "ultralight")]
pub mod ultralight;

/// Creation of new pages to be of a html type or a url
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum PageType {
    /// Allows visiting Url web pages
    Url(String),
    /// Allows custom html web pages
    Html(String),
}

/// Enables browser engines to display their images in different formats
pub enum PixelFormat {
    /// RGBA
    Rgba,
    /// BGRA
    Bgra,
}

/// Alias of usize used for controlling specific views
/// Only used by advanced to get views, basic simply uses u32
pub type ViewId = usize;

/// Trait to handle multiple browser engines
/// Currently only supports cpu renders via pixel_buffer
/// Passing a View id that does not exist will cause a panic
pub trait Engine {
    /// Used to do work in the actual browser engine
    fn update(&mut self);
    /// Has Ultralight perform a new render
    fn render(&mut self, size: Size<u32>);
    /// Request that the browser engine rerender a specific view that may have been updated
    fn request_render(&mut self, id: ViewId, size: Size<u32>);
    /// Creates new a new (possibly blank) view and returns the ViewId to interact with it
    fn new_view(&mut self, size: Size<u32>, content: Option<PageType>) -> ViewId;
    /// Removes desired view
    fn remove_view(&mut self, id: ViewId);

    /// Focuses webview
    fn focus(&mut self);
    /// Unfocuses webview
    fn unfocus(&self);
    /// Resizes webview
    fn resize(&mut self, size: Size<u32>);

    /// lets the engine handle keyboard events
    fn handle_keyboard_event(&mut self, id: ViewId, event: keyboard::Event);
    /// lets the engine handle mouse events
    fn handle_mouse_event(&mut self, id: ViewId, point: Point, event: mouse::Event);
    /// Handles Scrolles on view
    fn scroll(&mut self, id: ViewId, delta: mouse::ScrollDelta);

    /// Go to a specific page type
    fn goto(&mut self, id: ViewId, page_type: PageType);
    /// Refresh specific view
    fn refresh(&mut self, id: ViewId);
    /// Moves forward on view
    fn go_forward(&mut self, id: ViewId);
    /// Moves back on view
    fn go_back(&mut self, id: ViewId);

    /// Gets current url from view
    fn get_url(&self, id: ViewId) -> String;
    /// Gets current title from view
    fn get_title(&self, id: ViewId) -> String;
    /// Gets current cursor status from view
    fn get_cursor(&self, id: ViewId) -> Interaction;
    /// Gets cpu renderered webview
    fn get_view(&self, id: ViewId) -> &ImageInfo;
}
