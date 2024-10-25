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
    ChangeView(ViewId),
    CloseCurrentView,
    CloseView(ViewId),
    CreateView,
    GoBackward,
    GoForward,
    GoToUrl(Url),
    Refresh,
    SendKeyboardEvent(keyboard::Event),
    SendMouseEvent(mouse::Event, Point),
    Update,
    Resize(Size<u32>),
}

pub struct WebView<Engine, Message>
where
    Engine: engines::Engine,
{
    current_view: ViewId,
    on_url_change: Option<Box<dyn Fn(String) -> Message>>,
    url: String,
    on_title_change: Option<Box<dyn Fn(String) -> Message>>,
    title: String,
    webview: super::advanced::WebView<Engine, Message>,
}

impl<Engine: engines::Engine + Default, Message: Send + Clone + 'static> WebView<Engine, Message> {
    pub fn new() -> (Self, Task<Action>) {
        (
            WebView {
                current_view: 0,
                on_url_change: None,
                url: String::new(),
                on_title_change: None,
                title: String::new(),
                webview: super::advanced::WebView::new().0, // only keeps webview - we use basic task instead
            },
            Task::done(Action::CreateView),
        )
    }

    pub fn on_create_view(mut self, on_create_view: impl Fn(usize) -> Message + 'static) -> Self {
        self.webview.on_create_view = Some(Box::new(on_create_view));
        self
    }

    pub fn on_close_view(mut self, on_close_view: impl Fn(usize) -> Message + 'static) -> Self {
        self.webview.on_close_view = Some(Box::new(on_close_view));
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

        if let Some(url) = self.webview.engine.get_url(self.current_view) {
            if self.url != url {
                self.url = url;
            }
        }
        if let Some(title) = self.webview.engine.get_title(self.current_view) {
            if self.title != title {
                self.title = title;
            }
        }

        tasks.push(match action {
            Action::ChangeView(id) => {
                self.current_view = id;
                Task::none()
            }
            Action::CloseCurrentView => {
                self.webview.engine.remove_view(self.current_view);

                if let Some(on_view_close) = &self.webview.on_close_view {
                    Task::done((on_view_close)(self.current_view))
                } else {
                    Task::none()
                }
            }
            Action::CloseView(id) => {
                self.webview.engine.remove_view(id);

                if let Some(on_view_close) = &self.webview.on_close_view {
                    Task::done((on_view_close)(id))
                } else {
                    Task::none()
                }
            }
            Action::CreateView => {
                self.webview.engine.new_view(self.webview.view_size);
                Task::none()
            }
            Action::GoBackward => {
                self.webview.engine.go_back(self.current_view);
                Task::none()
            }
            Action::GoForward => {
                self.webview.engine.go_forward(self.current_view);
                Task::none()
            }
            Action::GoToUrl(url) => {
                self.webview
                    .engine
                    .goto(self.current_view, PageType::Url(url.to_string()));
                Task::none()
            }
            Action::Refresh => {
                self.webview.engine.refresh(self.current_view);
                Task::none()
            }
            Action::SendKeyboardEvent(event) => {
                self.webview
                    .engine
                    .handle_keyboard_event(self.current_view, event);
                Task::none()
            }
            Action::SendMouseEvent(point, event) => {
                self.webview
                    .engine
                    .handle_mouse_event(self.current_view, event, point);
                Task::none()
            }
            Action::Update => {
                self.webview.engine.update();
                self.webview.engine.render(self.webview.view_size);
                Task::none()
            }
            Action::Resize(size) => {
                self.webview.view_size = size;
                self.webview.engine.resize(size);
                Task::none()
            }
        });

        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    /// Returns webview element for the current view
    pub fn view(&self) -> Element<Action> {
        WebViewWidget::new(
            self.webview.view_size,
            self.webview.engine.get_view(self.current_view),
        )
        .into()
    }
}

struct WebViewWidget {
    bounds: Size<u32>,
    image: Image<Handle>,
}

impl WebViewWidget {
    fn new(bounds: Size<u32>, image: &ImageInfo) -> Self {
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
