use iced::keyboard;
use iced::mouse::{self, Interaction};
use iced::Size;
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

    fn do_work(&self);
    fn need_render(&self) -> bool;
    fn force_need_render(&self);
    fn render(&mut self);
    fn size(&self) -> Option<(u32, u32)>;
    fn resize(&mut self, size: Size<u32>);
    fn pixel_buffer(&mut self) -> Option<(PixelFormat, Vec<u8>)>;

    fn get_cursor(&self) -> Interaction;
    // fn get_icon(&self) -> Image<Handle>;
    fn goto_url(&self, url: &Url);
    fn goto_html(&self, html: &str);
    fn has_loaded(&self) -> Option<bool>;
    fn new_view(&mut self, page_type: PageType, size: Size<u32>) -> View<Self::Info>;
    fn get_views(&self) -> &Views<Self::Info>;
    fn get_views_mut(&mut self) -> &mut Views<Self::Info>;

    fn refresh(&self);
    fn go_forward(&self);
    fn go_back(&self);
    fn focus(&self);
    fn unfocus(&self);

    fn scroll(&self, delta: mouse::ScrollDelta);
    fn handle_keyboard_event(&self, event: keyboard::Event);
    fn handle_mouse_event(&mut self, point: Point, event: mouse::Event);
}

/// Engine specific view information
pub trait ViewInfo {
    fn url(&self) -> String;
    fn title(&self) -> String;
}

/// Can be converted from View to hold information for ResultType
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayView {
    pub id: usize,
    pub url: String,
    pub title: String,
}

impl<Info: ViewInfo> From<View<Info>> for DisplayView {
    fn from(view: View<Info>) -> Self {
        DisplayView {
            id: view.id,
            url: view.url(),
            title: view.title(),
        }
    }
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

    pub fn to_display_view(&self) -> DisplayView {
        DisplayView {
            id: self.id,
            url: self.url(),
            title: self.title(),
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

pub struct Views<Info: ViewInfo> {
    views: Vec<View<Info>>,
    history: Vec<usize>,
}

impl<Info: ViewInfo> Default for Views<Info> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Info: ViewInfo> Views<Info> {
    pub fn new() -> Self {
        Self {
            views: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn id_to_index(&self, id: usize) -> usize {
        for (idx, view) in self.views.iter().enumerate() {
            if view.id == id {
                return idx;
            }
        }
        panic!("Id: {} was not found", id);
    }

    pub fn index_to_id(&self, index: usize) -> usize {
        self.views
            .get(index)
            .unwrap_or_else(|| panic!("Index {} was not found", index))
            .id
    }

    pub fn get_current_id(&self) -> Option<usize> {
        Some(self.history.last()?.to_owned())
    }

    pub fn set_current_id(&mut self, id: usize) {
        self.history.push(id)
    }

    pub fn views(&self) -> &Vec<View<Info>> {
        &self.views
    }

    pub fn display_views(&self) -> Vec<DisplayView> {
        self.views
            .iter()
            .map(|view| view.to_display_view())
            .collect()
    }

    pub fn insert(&mut self, view: View<Info>) -> usize {
        let id = view.id;
        self.views.push(view);
        id
    }

    /// Returns the newly active view
    pub fn remove(&mut self, id: usize) -> Option<usize> {
        self.history.retain(|view_id| *view_id != id);

        self.views.retain(|view| view.id != id);
        self.get_current_id()
    }

    pub fn get_current(&self) -> Option<&View<Info>> {
        self.get(self.get_current_id()?)
    }

    pub fn get_current_mut(&mut self) -> Option<&mut View<Info>> {
        Some(self.get_mut(self.get_current_id()?))
    }

    pub fn get(&self, id: usize) -> Option<&View<Info>> {
        self.views.iter().find(|&view| view.id == id)
    }

    pub fn get_mut(&mut self, id: usize) -> &mut View<Info> {
        for view in self.views.iter_mut() {
            if view.id == id {
                return view;
            }
        }
        panic!("Unable to find view with id: {}", id);
    }
}

/// Allows users to create new views with url or custom html
#[derive(Clone)]
pub enum PageType {
    Url(&'static str),
    Html(&'static str),
}
