use super::{Engine, ViewId};
use crate::ImageInfo;
use blitz_dom::document::BaseDocument;
use blitz_renderer_vello::render_to_buffer;
pub use blitz_traits::ColorScheme;
use blitz_traits::Viewport;
use iced::Point;

pub struct View {
    id: ViewId,
    view: BaseDocument,
    last_frame: ImageInfo,
    cursor_pos: Point,
}

/// Implementation of the Blitz rendering engine for iced_webivew
pub struct Blitz {
    scale: f64,
    color_scheme: ColorScheme,
    views: Vec<View>,
}

impl Blitz {
    pub fn new(scale: f64, color_scheme: ColorScheme) -> Self {
        Blitz {
            scale,
            color_scheme,
            views: Vec::new(),
        }
    }

    fn get_view(&self, id: ViewId) -> &View {
        self.views
            .iter()
            .find(|&view| view.id == id)
            .expect("The requested View id was not found")
    }

    fn get_view_mut(&mut self, id: ViewId) -> &mut View {
        self.views
            .iter_mut()
            .find(|view| view.id == id)
            .expect("The requested View id was not found")
    }
}

impl Default for Blitz {
    fn default() -> Self {
        Self::new(1., ColorScheme::Light)
    }
}

impl Engine for Blitz {
    fn update(&mut self) {}

    async fn render(&mut self, size: iced::Size<u32>) {
        for view in self.views.iter_mut() {
            view.view.resolve();
            view.last_frame.pixels = render_to_buffer(
                &view.view,
                Viewport::new(
                    size.width,
                    size.height,
                    self.scale as f32,
                    self.color_scheme,
                ),
            )
            .await;
            view.last_frame.width = size.width;
            view.last_frame.height = size.height;
        }
    }

    async fn request_render(&mut self, id: super::ViewId, size: iced::Size<u32>) {
        let scale = self.scale.clone();
        let color_scheme = self.color_scheme.clone();
        let view = self.get_view_mut(id);
        view.last_frame.pixels = render_to_buffer(
            &view.view,
            Viewport::new(size.width, size.height, scale as f32, color_scheme),
        )
        .await;
        view.last_frame.width = size.width;
        view.last_frame.height = size.height;
    }

    fn new_view(
        &mut self,
        size: iced::Size<u32>,
        content: Option<super::PageType>,
    ) -> super::ViewId {
        let mut view = View {
            id: ViewId::new(),
            view: BaseDocument::new(Viewport::new(
                size.width,
                size.height,
                self.scale as f32,
                self.color_scheme,
            )),
            last_frame: ImageInfo::default(),
            cursor_pos: Point::default(),
        };

        if let Some(content) = content {
            match content {
                super::PageType::Url(url) => view.view.set_base_url(url.as_str()),
                super::PageType::Html(_html) => unimplemented!(),
            }
        };

        let id = view.id;
        self.views.push(view);
        id
    }

    fn remove_view(&mut self, id: super::ViewId) {
        self.views.retain(|view| view.id != id);
    }

    fn focus(&mut self) {}

    fn unfocus(&self) {}

    fn resize(&mut self, size: iced::Size<u32>) {
        for view in self.views.iter_mut() {
            view.view.set_viewport(Viewport::new(
                size.width,
                size.height,
                self.scale as f32,
                self.color_scheme,
            ));
        }
    }

    fn handle_keyboard_event(&mut self, id: super::ViewId, event: iced::keyboard::Event) {}

    fn handle_mouse_event(
        &mut self,
        id: super::ViewId,
        point: iced::Point,
        event: iced::mouse::Event,
    ) {
    }

    fn scroll(&mut self, id: super::ViewId, delta: iced::mouse::ScrollDelta) {}

    fn goto(&mut self, id: super::ViewId, page_type: super::PageType) {}

    fn refresh(&mut self, id: super::ViewId) {}

    fn go_forward(&mut self, id: super::ViewId) {}

    fn go_back(&mut self, id: super::ViewId) {}

    fn get_url(&self, id: super::ViewId) -> String {
        String::from("A.com")
    }

    fn get_title(&self, id: super::ViewId) -> String {
        String::from("A Title")
    }

    fn get_cursor(&self, id: super::ViewId) -> iced::mouse::Interaction {
        iced::mouse::Interaction::None
    }

    fn get_view(&self, id: super::ViewId) -> &crate::ImageInfo {
        &self.get_view(id).last_frame
    }
}
