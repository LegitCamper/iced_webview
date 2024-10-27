// simple engine used in test for webview
use super::Engine;
use crate::ImageInfo;
use rand::Rng;

#[derive(Default)]
pub struct Test {
    image: ImageInfo,
}

impl Engine for Test {
    fn update(&mut self) {}

    fn render(&mut self, size: iced::Size<u32>) {}

    fn request_render(&mut self, id: super::ViewId, size: iced::Size<u32>) {}

    fn new_view(&mut self, size: iced::Size<u32>) -> super::ViewId {
        rand::thread_rng().gen()
    }

    fn remove_view(&mut self, id: super::ViewId) {}

    fn focus(&mut self) {}

    fn unfocus(&self) {}

    fn resize(&mut self, size: iced::Size<u32>) {}

    fn handle_keyboard_event(&mut self, id: super::ViewId, event: iced::keyboard::Event) {}

    fn handle_mouse_event(
        &mut self,
        id: super::ViewId,
        point: iced::Point,
        event: iced::mouse::Event,
    ) {
    }

    fn goto(&mut self, id: super::ViewId, page_type: super::PageType) {}

    fn refresh(&mut self, id: super::ViewId) {}

    fn go_forward(&mut self, id: super::ViewId) {}

    fn go_back(&mut self, id: super::ViewId) {}

    fn scroll(&mut self, id: super::ViewId, delta: iced::mouse::ScrollDelta) {}

    fn get_url(&self, id: super::ViewId) -> Option<String> {
        None
    }

    fn get_title(&self, id: super::ViewId) -> Option<String> {
        None
    }

    fn get_view(&self, id: super::ViewId) -> &crate::ImageInfo {
        &self.image
    }

    fn get_cursor(&self, id: super::ViewId) -> iced::mouse::Interaction {
        iced::mouse::Interaction::Idle
    }
}
