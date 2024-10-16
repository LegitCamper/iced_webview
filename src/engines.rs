use iced::mouse::{self, Interaction};
use iced::{keyboard, Rectangle};
// use iced::widget::image::{Handle, Image};
use iced::Point;
use rand::Rng;
use url::Url;

use crate::ImageInfo;

#[cfg(feature = "ultralight")]
pub mod ultralight;

pub enum PixelFormat {
    Rgba,
    Bgra,
}

pub trait Engine {
    type Info: ViewInfo;
    const CANNOT_FIND_VIEW: &'static str = "Unable to get current view id";

    fn do_work(&self);
    fn need_render(&self, id: usize) -> bool;
    fn force_need_render(&self, id: usize);
    fn render(&mut self);
    /// Resizes all views
    fn resize(&mut self, size: Rectangle);
    fn pixel_buffer(&mut self, id: usize, size: Rectangle) -> (PixelFormat, Vec<u8>);

    fn get_cursor(&self, id: usize) -> Interaction;
    // fn get_icon(&self) -> Image<Handle>;
    fn goto_url(&self, id: usize, url: &Url);
    fn goto_html(&self, id: usize, html: &str);
    fn has_loaded(&self, id: usize) -> bool;
    fn new_view(&mut self, page_type: PageType, size: Rectangle) -> usize;
    fn get_views(&self) -> &Vec<View<Self::Info>>;
    fn get_views_mut(&mut self) -> &mut Vec<View<Self::Info>>;

    fn refresh(&self, id: usize);
    fn go_forward(&self, id: usize);
    fn go_back(&self, id: usize);
    fn focus(&self, id: usize);
    fn unfocus(&self, id: usize);

    fn scroll(&self, id: usize, delta: mouse::ScrollDelta);
    fn handle_keyboard_event(&self, id: usize, event: keyboard::Event);
    fn handle_mouse_event(&mut self, id: usize, point: Point, event: mouse::Event);
}

/// Engine specific view information
pub trait ViewInfo {
    fn url(&self) -> String;
    fn title(&self) -> String;
}

/// Stores view info like url & title
pub struct View<Info: ViewInfo> {
    id: usize,
    view: ImageInfo,
    info: Info,
}

impl<Info: ViewInfo> View<Info> {
    pub fn new(info: Info) -> Self {
        let id = rand::thread_rng().gen();
        Self {
            id,
            view: ImageInfo::default(),
            info,
        }
    }

    pub fn get_view(&self) -> &ImageInfo {
        &self.view
    }

    pub fn set_view(&mut self, view: ImageInfo) {
        self.view = view;
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn url(&self) -> String {
        self.info.url()
    }

    pub fn title(&self) -> String {
        self.info.title()
    }
}

/// Allows users to create new views with url or custom html
#[derive(Clone)]
pub enum PageType {
    Url(&'static str),
    Html(&'static str),
}
