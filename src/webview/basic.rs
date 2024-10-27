use iced::advanced::{
    self,
    graphics::core::event,
    layout,
    renderer::{self},
    widget::Tree,
    Clipboard, Layout, Shell, Widget,
};
use iced::event::Status;
use iced::keyboard;
use iced::widget::image::{Handle, Image};
use iced::{mouse, Element, Point, Size, Task};
use iced::{theme::Theme, Event, Length, Rectangle};
use url::Url;

use crate::{engines, ImageInfo, PageType, ViewId};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Changes view to the desired view index
    ChangeView(u32),
    /// Closes current window & makes last used view the current one
    CloseCurrentView,
    /// Closes specific view id
    CloseView(u32),
    /// Creates a new view and makes its index view + 1
    CreateView,
    GoBackward,
    GoForward,
    GoToUrl(Url),
    Refresh,
    SendKeyboardEvent(keyboard::Event),
    SendMouseEvent(mouse::Event, Point),
    /// Allows users to control when the browser engine proccesses interactions in subscriptions
    Update,
    Resize(Size<u32>),
}

pub struct WebView<Engine, Message>
where
    Engine: engines::Engine,
{
    current_view: Option<u32>,
    view_ids: Vec<ViewId>, // allow users to index by simple id like 0 or 1 instead of a true id
    on_close_view: Option<Message>,
    on_create_view: Option<Message>,
    on_url_change: Option<Box<dyn Fn(String) -> Message>>,
    url: String,
    on_title_change: Option<Box<dyn Fn(String) -> Message>>,
    title: String,
    webview: super::advanced::WebView<Engine, Message>,
}

impl<Engine: engines::Engine + Default, Message: Send + Clone + 'static> WebView<Engine, Message> {
    fn get_view_index(&self) -> Option<usize> {
        Some(*self.view_ids.get(self.current_view? as usize)?)
    }

    fn index_to_view_id(&self, index: u32) -> usize {
        *self
            .view_ids
            .get(index as usize)
            .expect("Failed to find that index, maybe its already been closed?")
    }
}

impl<Engine: engines::Engine + Default, Message: Send + Clone + 'static> WebView<Engine, Message> {
    pub fn new() -> Self {
        WebView {
            current_view: None,
            view_ids: Vec::new(),
            on_close_view: None,
            on_create_view: None,
            on_url_change: None,
            url: String::new(),
            on_title_change: None,
            title: String::new(),
            webview: super::advanced::WebView::new(),
        }
    }

    pub fn on_create_view(mut self, on_create_view: Message) -> Self {
        self.on_create_view = Some(on_create_view);
        self
    }

    pub fn on_close_view(mut self, on_close_view: Message) -> Self {
        self.on_close_view = Some(on_close_view);
        self
    }

    pub fn on_url_change(mut self, on_url_change: impl Fn(String) -> Message + 'static) -> Self {
        self.on_url_change = Some(Box::new(on_url_change));
        self
    }

    pub fn on_title_change(
        mut self,
        on_title_change: impl Fn(String) -> Message + 'static,
    ) -> Self {
        self.on_title_change = Some(Box::new(on_title_change));
        self
    }

    /// Passes update to webview
    pub fn update(&mut self, action: Action) -> Task<Message> {
        let mut tasks = Vec::new();

        if let Some(_) = self.current_view {
            if let Some(view_index) = self.get_view_index() {
                if let Some(on_url_change) = &self.on_url_change {
                    if let Some(url) = self.webview.engine.get_url(view_index) {
                        if self.url != url {
                            self.url = url.clone();
                            tasks.push(Task::done(on_url_change(url)))
                        }
                    }
                }
                if let Some(on_title_change) = &self.on_title_change {
                    if let Some(title) = self.webview.engine.get_title(view_index) {
                        if self.title != title {
                            self.title = title.clone();
                            tasks.push(Task::done(on_title_change(title)))
                        }
                    }
                }
            }
        }

        match action {
            Action::ChangeView(index) => {
                self.current_view = Some(index);
            }
            Action::CloseCurrentView => {
                let Some(view_index) = self.get_view_index() else {
                    return Task::batch(tasks);
                };
                self.webview.engine.remove_view(view_index);
                self.view_ids.remove(view_index);
                if let Some(on_view_close) = &self.on_close_view {
                    tasks.push(Task::done(on_view_close.clone()));
                }
            }
            Action::CloseView(index) => {
                self.webview
                    .engine
                    .remove_view(self.index_to_view_id(index));
                self.view_ids.remove(self.index_to_view_id(index));

                if let Some(on_view_close) = &self.on_close_view {
                    tasks.push(Task::done(on_view_close.clone()))
                }
            }
            Action::CreateView => {
                self.view_ids
                    .push(self.webview.engine.new_view(self.webview.view_size));

                if let Some(on_view_create) = &self.on_create_view {
                    tasks.push(Task::done(on_view_create.clone()))
                }
            }
            Action::GoBackward => {
                let Some(view_index) = self.get_view_index() else {
                    return Task::batch(tasks);
                };
                self.webview.engine.go_back(view_index);
            }
            Action::GoForward => {
                let Some(view_index) = self.get_view_index() else {
                    return Task::batch(tasks);
                };
                self.webview.engine.go_forward(view_index);
            }
            Action::GoToUrl(url) => {
                let Some(view_index) = self.get_view_index() else {
                    return Task::batch(tasks);
                };
                self.webview
                    .engine
                    .goto(view_index, PageType::Url(url.to_string()));
            }
            Action::Refresh => {
                let Some(view_index) = self.get_view_index() else {
                    return Task::batch(tasks);
                };
                self.webview.engine.refresh(view_index);
            }
            Action::SendKeyboardEvent(event) => {
                let Some(view_index) = self.get_view_index() else {
                    return Task::batch(tasks);
                };
                self.webview.engine.handle_keyboard_event(view_index, event);
            }
            Action::SendMouseEvent(point, event) => {
                let Some(view_index) = self.get_view_index() else {
                    return Task::batch(tasks);
                };
                self.webview
                    .engine
                    .handle_mouse_event(view_index, event, point);
            }
            Action::Update => {
                self.webview.engine.update();
                self.webview.engine.render(self.webview.view_size);
            }
            Action::Resize(size) => {
                self.webview.view_size = size;
                self.webview.engine.resize(size);
            }
        };

        Task::batch(tasks)
    }

    /// Returns webview element for the current view
    pub fn view(&self) -> Element<Action> {
        match self.get_view_index() {
            Some(view_index) => WebViewWidget::with(
                self.webview.view_size,
                self.webview.engine.get_view(view_index),
            ),
            None => WebViewWidget::with(self.webview.view_size, &ImageInfo::default()),
        }
        .into()
    }
}

struct WebViewWidget {
    bounds: Size<u32>,
    image: Image<Handle>,
}

impl WebViewWidget {
    fn with(bounds: Size<u32>, image: &ImageInfo) -> Self {
        Self {
            bounds,
            image: image.as_image(),
        }
    }
}

impl<Renderer> Widget<Action, Theme, Renderer> for WebViewWidget
where
    Renderer: iced::advanced::image::Renderer<Handle = iced::advanced::image::Handle>,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        <Image<Handle> as Widget<Action, Theme, Renderer>>::draw(
            &self.image,
            tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        )
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Action>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let size = Size::new(layout.bounds().width as u32, layout.bounds().height as u32);
        if self.bounds != size {
            shell.publish(Action::Resize(size));
        }

        match event {
            Event::Keyboard(event) => {
                shell.publish(Action::SendKeyboardEvent(event));
            }
            Event::Mouse(event) => {
                if let Some(point) = cursor.position_in(layout.bounds()) {
                    shell.publish(Action::SendMouseEvent(event, point));
                }
            }
            _ => (),
        }
        Status::Ignored
    }
}

impl<'a, Message: 'a, Renderer> From<WebViewWidget> for Element<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + advanced::image::Renderer<Handle = advanced::image::Handle>,
    WebViewWidget: Widget<Message, Theme, Renderer>,
{
    fn from(widget: WebViewWidget) -> Self {
        Self::new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Message {}

    #[test]
    fn test_view_interactions() {
        // using ultralight here crashes?
        let mut webview: WebView<engines::test::Test, Message> = WebView::new();
        let _ = webview.update(Action::CreateView);
        assert_eq!(webview.view_ids.len(), 1);
        let _ = webview.update(Action::ChangeView(0));
        let _ = webview.update(Action::CloseCurrentView);

        assert_eq!(webview.current_view, None);
        assert_eq!(webview.view_ids.len(), 0);
    }
}
