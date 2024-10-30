use futures::executor::block_on;
use gosub_css3::system::Css3System;
use gosub_html5::document::document_impl::DocumentImpl;
use gosub_html5::parser::Html5Parser;
use gosub_render_backend::{Point, SizeU32, FP};
use gosub_renderer::draw::SceneDrawer;
use gosub_renderer::render_tree::TreeDrawer;
use gosub_rendering::render_tree::RenderTree;
use gosub_shared::types::{Result, Size};
use gosub_taffy::TaffyLayouter;
use gosub_useragent::application::{Application, CustomEventInternal, WindowOptions};
use gosub_useragent::tabs::{self, Tab};
use gosub_vello::VelloBackend;
use slotmap::DefaultKey;
use std::collections::HashMap;
use url::Url;

use crate::ImageInfo;

use super::{Engine, PageType, PixelFormat, ViewId};

type Backend = VelloBackend;
type Layouter = TaffyLayouter;
type CssSystem = Css3System;
type Document = DocumentImpl<CssSystem>;
type HtmlParser<'a> = Html5Parser<'a, Document, CssSystem>;
type Drawer = TreeDrawer<Backend, Layouter, Document, CssSystem>;
type Tree = RenderTree<Layouter, CssSystem>;

pub struct GoSub {
    debug: bool,
    id_to_slot: HashMap<usize, DefaultKey>,
    layouter: Layouter,
    tabs: tabs::Tabs<Drawer, Backend, Layouter, Tree, Document, CssSystem>,
    backend: VelloBackend,
    view_cache: Vec<ImageInfo>,
    size: Size<u32>,
}

impl GoSub {
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            id_to_slot: HashMap::new(),
            layouter: TaffyLayouter,
            backend: VelloBackend {},
            tabs: tabs::Tabs::default(),
            view_cache: Vec::new(),
            size: Size::new(1920, 1080),
        }
    }
}

impl Engine for GoSub {
    fn update(&mut self) {
        todo!()
    }

    fn render(&mut self, _size: iced::Size<u32>) {
        for (_, slot) in self.id_to_slot.iter() {
            if let Some(tab) = self.tabs.tabs.get_mut(*slot) {
                tab.data.set_needs_redraw();
                // tab.data.get_img_cache().lock().unwrap();
                // get view?
            }
        }
    }

    fn request_render(&mut self, id: ViewId, _size: iced::Size<u32>) -> Option<()> {
        let key = self.id_to_slot.get(&id)?;
        self.tabs
            .tabs
            .get_mut(*key)
            .unwrap()
            .data
            .set_needs_redraw();
        Some(())
    }

    fn new_view(&mut self, size: iced::Size<u32>) -> Option<ViewId> {
        todo!()
    }

    fn remove_view(&mut self, id: ViewId) -> Option<()> {
        todo!()
    }

    fn focus(&mut self) {
        todo!()
    }

    fn unfocus(&self) {
        todo!()
    }

    fn resize(&mut self, size: iced::Size<u32>) {
        todo!()
    }

    fn handle_keyboard_event(&mut self, id: ViewId, event: iced::keyboard::Event) -> Option<()> {
        todo!()
    }

    fn handle_mouse_event(
        &mut self,
        id: ViewId,
        point: iced::Point,
        event: iced::mouse::Event,
    ) -> Option<()> {
        todo!()
    }

    fn goto(&mut self, id: ViewId, page_type: PageType) -> Option<()> {
        todo!()
    }

    fn refresh(&mut self, id: ViewId) -> Option<()> {
        todo!()
    }

    fn go_forward(&mut self, id: ViewId) -> Option<()> {
        todo!()
    }

    fn go_back(&mut self, id: ViewId) -> Option<()> {
        todo!()
    }

    fn scroll(&mut self, id: ViewId, delta: iced::mouse::ScrollDelta) -> Option<()> {
        todo!()
    }

    fn get_url(&self, id: ViewId) -> super::EngineResult<String> {
        todo!()
    }

    fn get_title(&self, id: ViewId) -> super::EngineResult<String> {
        todo!()
    }

    fn get_cursor(&self, id: ViewId) -> super::EngineResult<iced::mouse::Interaction> {
        todo!()
    }

    fn get_view(&self, id: ViewId) -> super::EngineResult<&ImageInfo> {
        todo!()
    }
}

// impl Engine for GoSub {
//     fn do_work(&self) {
//         todo!()
//     }

//     fn need_render(&self, id: usize) -> bool {
//         false
//     }

//     fn force_render(&mut self, id: usize) {

//     }

//     fn render(&mut self, id: usize) {

//     }

//     fn resize(&mut self, size: iced::Size<u32>) {
//         todo!()
//     }

//     fn pixel_buffer(&mut self, id: usize) -> Option<(PixelFormat, Vec<u8>)> {
//         let key = self.id_to_slot.get(&id).expect("Failed to get key with id");
//         let pos = self
//             .tabs
//             .tabs
//             .iter()
//             .position(|tab| tab.0 == *key)
//             .expect("Failed to get tab pos");
//         Some(self.view_cache.get(pos).unwrap())
//     }

//     fn get_cursor(&self, id: usize) -> iced::mouse::Interaction {
//         todo!()
//     }

//     fn goto_url(&self, id: usize, url: &Url) {
//         todo!()
//     }

//     fn goto_html(&self, id: usize, html: &str) {
//         todo!()
//     }

//     fn has_loaded(&self, id: usize) -> Option<bool> {
//         todo!()
//     }

//     fn new_view(&mut self, page_type: PageType, size: iced::Size<u32>) -> usize {
//         // if let PageType::Url(url) = page_type {
//         //     let tab = block_on(
//         //         Tab::from_url::<Html5Parser<CssSystem, Document<CssSystem>>>(
//         //             Url::parse(url).expect("Failed to parse string as url"),
//         //             self.layouter,
//         //             true,
//         //         ),
//         //     )
//         //     .expect("Failed to create new tab");
//         //     let key = self.tabs.tabs.insert(tab);
//         //     let id = rand::thread_rng().gen();
//         //     self.id_to_slot.insert(id, key);
//         //     return id;
//         // } else {
//         //     unimplemented!()
//         // }
//         todo!()
//     }

//     fn remove_view(&mut self, id: usize) {
//         let key = self
//             .id_to_slot
//             .remove(&id)
//             .expect("Failed to retreive tab id");
//         self.tabs.tabs.remove(key).expect("Failed to remove tab");
//     }

//     fn get_views(&self) -> Vec<View> {
//         self.tabs
//             .tabs
//             .iter()
//             .zip(self.view_cache.iter())
//             .map(|(tab, image)| super::View {
//                 title: tab.1.title.clone(),
//                 url: tab.1.url.to_string(),
//                 view: image.clone(),
//             })
//             .collect()
//     }

//     fn get_view(&self, id: usize) -> View {
//         let key = self.id_to_slot.get(&id).expect("Failed to get slot id");
//         for ((_, tab), image) in self
//             .tabs
//             .tabs
//             .iter()
//             .zip(self.view_cache.iter())
//             .filter(|((iter_key, _), _)| iter_key == key)
//         {
//             return super::View {
//                 title: tab.title.clone(),
//                 url: tab.url.to_string(),
//                 view: image.clone(),
//             };
//         }
//         panic!("Could not get view")
//     }

//     fn refresh(&self, id: usize) {
//         // let key = self.id_to_slot.get(&id).expect("Failed to get slot id");
//         // self.tabs.tabs.get(*key).unwrap();
//         unimplemented!() // not implemented in gosub yet
//     }

//     fn go_forward(&self, id: usize) {
//         unimplemented!() // not implemented in gosub yet
//     }

//     fn go_back(&self, id: usize) {
//         unimplemented!() // not implemented in gosub yet
//     }

//     fn focus(&self, id: usize) {
//         unimplemented!() // not implemented in gosub yet
//     }

//     fn unfocus(&self, id: usize) {
//         unimplemented!() // not implemented in gosub yet
//     }

//     fn scroll(&mut self, id: usize, delta: iced::mouse::ScrollDelta) {
//         let Some(tab) = self.tabs.get_current_tab() else {
//             return;
//         };

//         let delta = match delta {
//             iced::mouse::ScrollDelta::Pixels { x, y } => (x as f32, y as f32),
//             iced::mouse::ScrollDelta::Lines { x, y } => (x * 4.0, y * 12.0),
//         };

//         let delta = Point::new(delta.0 as FP, delta.1 as FP);

//         tab.data.scroll(delta);
//     }

//     fn handle_keyboard_event(&mut self, id: usize, event: iced::keyboard::Event) {
//         let key = self.id_to_slot.get(&id).expect("Failed to get slot id");
//         let Some(tab) = self.tabs.tabs.get_mut(*key) else {
//             return;
//         };

//         match event {
//             iced::keyboard::Event::KeyPressed {
//                 key,
//                 modified_key: _,
//                 physical_key: _,
//                 location: _,
//                 modifiers: _,
//                 text: _,
//             } => {
//                 if key == iced::keyboard::Key::Character("d".into()) {
//                     tab.data.toggle_debug();
//                 } else if key == iced::keyboard::Key::Character("k".into()) {
//                     tab.data.clear_buffers();
//                 }
//             }
//             iced::keyboard::Event::KeyReleased {
//                 key: _,
//                 location: _,
//                 modifiers: _,
//             } => (),
//             iced::keyboard::Event::ModifiersChanged(_modifiers) => unimplemented!(),
//         }
//     }

//     fn handle_mouse_event(&mut self, id: usize, point: iced::Point, _event: iced::mouse::Event) {
//         let key = self.id_to_slot.get(&id).expect("Failed to get slot id");
//         let Some(tab) = self.tabs.tabs.get_mut(*key) else {
//             return;
//         };

//         tab.data
//             .mouse_move(&mut self.backend, point.x as FP, point.y as FP);
//     }
// }
