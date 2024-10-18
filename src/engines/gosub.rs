use gosub_css3::system::Css3System;
use gosub_html5::document::document_impl::DocumentImpl;
use gosub_html5::parser::Html5Parser;
use gosub_render_backend::{Point, SizeU32, FP};
use gosub_renderer::draw::SceneDrawer;
use gosub_renderer::render_tree::TreeDrawer;
use gosub_rendering::render_tree::RenderTree;
use gosub_shared::types::Result;
use gosub_taffy::TaffyLayouter;
use gosub_useragent::application::{Application, CustomEventInternal, WindowOptions};
use gosub_useragent::tabs::{self, Tab};
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
    layouter: Layouter,
    tabs: tabs::Tabs<Drawer, Backend, Layouter, Tree, Document, CssSystem>,
    backend: VelloBackend,
}

impl GoSub {
    pub fn new() -> Self {
        Self {
            layouter: TaffyLayouter,
            backend: VelloBackend {},
            tabs: tabs::Tabs::default(),
        }
    }
}

impl Engine for GoSub {
    fn do_work(&self) {
        todo!()
    }

    fn need_render(&self, id: slotmap::DefaultKey) -> bool {
        todo!()
    }

    fn force_render(&self, id: slotmap::DefaultKey) {}

    fn render(&mut self, id: slotmap::DefaultKey) {
        todo!()
    }

    fn resize(&mut self, size: iced::Size<u32>) {
        todo!()
    }

    fn pixel_buffer(&mut self, id: slotmap::DefaultKey) -> Option<(PixelFormat, Vec<u8>)> {
        todo!()
    }

    fn get_cursor(&self, id: slotmap::DefaultKey) -> iced::mouse::Interaction {
        todo!()
    }

    fn goto_url(&self, id: slotmap::DefaultKey, url: &Url) {
        todo!()
    }

    fn goto_html(&self, id: slotmap::DefaultKey, html: &str) {
        todo!()
    }

    fn has_loaded(&self, id: slotmap::DefaultKey) -> Option<bool> {
        todo!()
    }

    fn new_view(&mut self, page_type: PageType, size: iced::Size<u32>) -> slotmap::DefaultKey {
        if let PageType::Url(url) = page_type {
            self.tabs.tabs.insert(Tab::from_url(
                Url::parse(url).unwrap(),
                self.layouter,
                false,
            ))
        } else {
            unimplemented!()
        }
    }

    fn remove_view(&mut self, id: slotmap::DefaultKey) {
        todo!()
    }

    fn get_views(&self) -> Vec<View> {
        todo!()
    }

    fn get_view(&self, id: slotmap::DefaultKey) -> View {
        todo!()
    }

    fn refresh(&self, id: slotmap::DefaultKey) {
        todo!()
    }

    fn go_forward(&self, id: slotmap::DefaultKey) {
        todo!()
    }

    fn go_back(&self, id: slotmap::DefaultKey) {
        todo!()
    }

    fn focus(&self, id: slotmap::DefaultKey) {
        todo!()
    }

    fn unfocus(&self, id: slotmap::DefaultKey) {
        todo!()
    }

    fn scroll(&mut self, id: slotmap::DefaultKey, delta: iced::mouse::ScrollDelta) {
        let Some(tab) = self.tabs.get_current_tab() else {
            return;
        };

        let delta = match delta {
            iced::mouse::ScrollDelta::Pixels { x, y } => (x as f32, y as f32),
            iced::mouse::ScrollDelta::Lines { x, y } => (x * 4.0, y * 12.0),
        };

        let delta = Point::new(delta.0 as FP, delta.1 as FP);

        tab.data.scroll(delta);
    }

    fn handle_keyboard_event(&mut self, id: slotmap::DefaultKey, event: iced::keyboard::Event) {
        let Some(tab) = self.tabs.tabs.get_mut(id) else {
            return;
        };

        match event {
            iced::keyboard::Event::KeyPressed {
                key,
                modified_key: _,
                physical_key: _,
                location: _,
                modifiers: _,
                text: _,
            } => {
                if key == iced::keyboard::Key::Character("d".into()) {
                    tab.data.toggle_debug();
                } else if key == iced::keyboard::Key::Character("k".into()) {
                    tab.data.clear_buffers();
                }
            }
            iced::keyboard::Event::KeyReleased {
                key: _,
                location: _,
                modifiers: _,
            } => (),
            iced::keyboard::Event::ModifiersChanged(_modifiers) => unimplemented!(),
        }
    }

    fn handle_mouse_event(
        &mut self,
        id: slotmap::DefaultKey,
        point: iced::Point,
        _event: iced::mouse::Event,
    ) {
        let Some(tab) = self.tabs.tabs.get_mut(id) else {
            return;
        };

        tab.data
            .mouse_move(&mut self.backend, point.x as FP, point.y as FP);
    }
}
