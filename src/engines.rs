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

/// Result type for get_url, get_title, get_view, and get_cursor
/// This is because they can fail by the wrong id, and the requested view, may not be loaded yet
#[derive(Debug, Clone)]
pub enum EngineResult<T> {
    IdDoesNotExist,
    NotLoaded,
    Success(T),
}

/// Alias of usize used for controlling specific views
/// Only used by advanced to get views, basic simply uses u32
pub type ViewId = usize;

/// Trait to handle multiple browser engines
/// Currently only supports cpu renders via pixel_buffer
pub trait Engine {
    /// Used to do work in the actual browser engine
    fn update(&mut self);
    /// Has Ultralight perform a new render
    fn render(&mut self, size: Size<u32>);
    /// Request that the browser engine rerender a specific view that may have been updated
    /// Can fail if requested id does not exist
    fn request_render(&mut self, id: ViewId, size: Size<u32>) -> Option<()>;
    /// Creates new a new blank view and returns the ViewId to interact with it
    /// Can fail if underlying engine fails to create view
    fn new_view(&mut self, size: Size<u32>) -> Option<ViewId>;
    /// Removes desired view
    /// Can fail if requested id does not exist
    fn remove_view(&mut self, id: ViewId) -> Option<()>;

    // window changes - no id needed they work for all views(gloabally)
    fn focus(&mut self);
    fn unfocus(&self);
    fn resize(&mut self, size: Size<u32>);

    // handle events per engine
    /// Can fail if requested id does not exist
    fn handle_keyboard_event(&mut self, id: ViewId, event: keyboard::Event) -> Option<()>;
    /// Can fail if requested id does not exist
    fn handle_mouse_event(&mut self, id: ViewId, point: Point, event: mouse::Event) -> Option<()>;

    /// Allows navigating to html or Url on a specific view
    /// Can fail if requested id does not exist
    fn goto(&mut self, id: ViewId, page_type: PageType) -> Option<()>;
    /// Can fail if requested id does not exist
    fn refresh(&mut self, id: ViewId) -> Option<()>;
    /// Can fail if requested id does not exist
    fn go_forward(&mut self, id: ViewId) -> Option<()>;
    /// Can fail if requested id does not exist
    fn go_back(&mut self, id: ViewId) -> Option<()>;
    /// Can fail if requested id does not exist
    fn scroll(&mut self, id: ViewId, delta: mouse::ScrollDelta) -> Option<()>;

    /// Can fail if requested id does not exist or page has not loaded and therfore has no url
    fn get_url(&self, id: ViewId) -> EngineResult<String>;
    /// Can fail if requested id does not exist or page has not loaded and therfore has no title
    fn get_title(&self, id: ViewId) -> EngineResult<String>;
    /// Can fail if requested id does not exist
    fn get_cursor(&self, id: ViewId) -> EngineResult<Interaction>;
    /// Can fail if requested id does not exist
    fn get_view(&self, id: ViewId) -> EngineResult<&ImageInfo>;
}

/// Allows users to create new views with url or custom html
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum PageType {
    Url(String),
    Html(String),
}
