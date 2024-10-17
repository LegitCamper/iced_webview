use gosub_css3::system::Css3System;
use gosub_html5::document::document_impl::DocumentImpl;
use gosub_html5::parser::Html5Parser;
use gosub_renderer::draw::SceneDrawer;
use gosub_renderer::render_tree::TreeDrawer;
use gosub_rendering::render_tree::RenderTree;
use gosub_shared::types::Result;
use gosub_taffy::TaffyLayouter;
use gosub_useragent::application::{Application, CustomEventInternal, WindowOptions};
use gosub_useragent::tabs;
use gosub_vello::VelloBackend;
use url::Url;

use super::{Engine, PageType, PixelFormat, View};

type Backend = VelloBackend;
type Layouter = TaffyLayouter;
type CssSystem = Css3System;
type Document = DocumentImpl<CssSystem>;
type HtmlParser<'a> = Html5Parser<'a, Document, CssSystem>;
type Drawer = TreeDrawer<Backend, Layouter, Document, CssSystem>;
type Tree = RenderTree<Layouter, CssSystem>;

pub struct GoSub {
    tabs: tabs::Tabs<Drawer, Backend, Layouter, Tree, Document, CssSystem>,
}

impl GoSub {
    pub fn new() -> Self {
        Self {
            tabs: tabs::Tabs::default(),
        }
    }
}

impl Engine for GoSub {
    fn do_work(&self) {
        todo!()
    }

    fn need_render(&self) -> bool {
        todo!()
    }

    fn force_render(&self) {
        todo!()
    }

    fn render(&mut self) {
        todo!()
    }

    fn size(&self) -> Option<(u32, u32)> {
        todo!()
    }

    fn resize(&mut self, size: iced::Size<u32>) {
        todo!()
    }

    fn pixel_buffer(&mut self) -> Option<(PixelFormat, Vec<u8>)> {
        todo!()
    }

    fn get_cursor(&self) -> iced::mouse::Interaction {
        todo!()
    }

    fn goto_url(&self, url: &Url) {
        todo!()
    }

    fn goto_html(&self, html: &str) {
        todo!()
    }

    fn has_loaded(&self) -> Option<bool> {
        todo!()
    }

    fn new_view(&mut self, page_type: PageType, size: iced::Size<u32>) -> usize {
        todo!()
    }

    fn get_views(&self) -> Vec<View> {
        todo!()
    }

    fn get_view(&self, id: usize) -> View {
        todo!()
    }

    fn refresh(&self) {
        todo!()
    }

    fn go_forward(&self) {
        todo!()
    }

    fn go_back(&self) {
        todo!()
    }

    fn focus(&self) {
        todo!()
    }

    fn unfocus(&self) {
        todo!()
    }

    fn scroll(&self, delta: iced::mouse::ScrollDelta) {
        todo!()
    }

    fn handle_keyboard_event(&self, event: iced::keyboard::Event) {
        todo!()
    }

    fn handle_mouse_event(&mut self, point: iced::Point, event: iced::mouse::Event) {
        todo!()
    }
}
