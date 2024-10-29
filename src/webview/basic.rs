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
use iced::mouse::{self, Interaction};
use iced::widget::image::{Handle, Image};
use iced::{theme::Theme, Event, Length, Rectangle};
use iced::{Element, Point, Size, Task};
use url::Url;

use crate::{engines, ImageInfo, PageType, ViewId};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Changes view to the desired view index
    ChangeView(u32),
    /// Closes current window & makes last used view the current one
    CloseCurrentView,
    /// Closes specific view index
    CloseView(u32),
    /// Creates a new view and makes its index view + 1
    CreateView(PageType),
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
    engine: Engine,
    view_size: Size<u32>,
    current_view_index: Option<usize>, // the index corresponding to the view_ids list of ViewIds
    view_ids: Vec<ViewId>, // allow users to index by simple id like 0 or 1 instead of a true id
    on_close_view: Option<Message>,
    on_create_view: Option<Message>,
    on_url_change: Option<Box<dyn Fn(String) -> Message>>,
    url: String,
    on_title_change: Option<Box<dyn Fn(String) -> Message>>,
    title: String,
}

impl<Engine: engines::Engine + Default, Message: Send + Clone + 'static> WebView<Engine, Message> {
    fn get_current_view_id(&self) -> ViewId {
        *self
            .view_ids
            .get(self.current_view_index.expect(
                "The current view index is not currently set. Ensure you call the Action prior",
            ))
            .expect("Could find view index for current view. Maybe its already been closed?")
    }

    fn index_as_view_id(&self, index: u32) -> usize {
        *self
            .view_ids
            .get(index as usize)
            .expect("Failed to find that index, maybe its already been closed?")
    }
}

impl<Engine: engines::Engine + Default, Message: Send + Clone + 'static> Default
    for WebView<Engine, Message>
{
    fn default() -> Self {
        WebView {
            engine: Engine::default(),
            view_size: Size {
                width: 1920,
                height: 1080,
            },
            current_view_index: None,
            view_ids: Vec::new(),
            on_close_view: None,
            on_create_view: None,
            on_url_change: None,
            url: String::new(),
            on_title_change: None,
            title: String::new(),
        }
    }
}

impl<Engine: engines::Engine + Default, Message: Send + Clone + 'static> WebView<Engine, Message> {
    pub fn new() -> Self {
        Self::default()
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

        if self.current_view_index.is_some() {
            if let Some(on_url_change) = &self.on_url_change {
                if let Some(url) = self.engine.get_url(self.get_current_view_id()) {
                    if self.url != url {
                        self.url = url.clone();
                        tasks.push(Task::done(on_url_change(url)))
                    }
                }
            }
            if let Some(on_title_change) = &self.on_title_change {
                if let Some(title) = self.engine.get_title(self.get_current_view_id()) {
                    if self.title != title {
                        self.title = title.clone();
                        tasks.push(Task::done(on_title_change(title)))
                    }
                }
            }
        }

        match action {
            Action::ChangeView(index) => {
                self.current_view_index = Some(index as usize);
            }
            Action::CloseCurrentView => {
                self.engine.remove_view(self.get_current_view_id());
                self.view_ids.remove(self.get_current_view_id());
                if let Some(on_view_close) = &self.on_close_view {
                    tasks.push(Task::done(on_view_close.clone()));
                }
            }
            Action::CloseView(index) => {
                self.engine.remove_view(self.index_as_view_id(index));
                self.view_ids.remove(self.index_as_view_id(index));

                if let Some(on_view_close) = &self.on_close_view {
                    tasks.push(Task::done(on_view_close.clone()))
                }
            }
            Action::CreateView(page_type) => {
                let id = self
                    .engine
                    .new_view(self.view_size)
                    .expect("Failed to create new view");
                self.view_ids.push(id);
                self.engine
                    .goto(id, page_type)
                    .expect("Failed to page type");

                if let Some(on_view_create) = &self.on_create_view {
                    tasks.push(Task::done(on_view_create.clone()))
                }
            }
            Action::GoBackward => {
                self.engine.go_back(self.get_current_view_id());
                self.engine
                    .request_render(self.get_current_view_id(), self.view_size);
            }
            Action::GoForward => {
                self.engine.go_forward(self.get_current_view_id());
                self.engine
                    .request_render(self.get_current_view_id(), self.view_size);
            }
            Action::GoToUrl(url) => {
                self.engine
                    .goto(self.get_current_view_id(), PageType::Url(url.to_string()));
                self.engine
                    .request_render(self.get_current_view_id(), self.view_size);
            }
            Action::Refresh => {
                self.engine.refresh(self.get_current_view_id());
                self.engine
                    .request_render(self.get_current_view_id(), self.view_size);
            }
            Action::SendKeyboardEvent(event) => {
                self.engine
                    .handle_keyboard_event(self.get_current_view_id(), event);
                self.engine
                    .request_render(self.get_current_view_id(), self.view_size);
            }
            Action::SendMouseEvent(point, event) => {
                self.engine
                    .handle_mouse_event(self.get_current_view_id(), event, point);
                self.engine
                    .request_render(self.get_current_view_id(), self.view_size);
            }
            Action::Update => {
                self.engine.update();
                self.engine.render(self.view_size);
            }
            Action::Resize(size) => {
                self.view_size = size;
                self.engine.resize(size);
            }
        };

        Task::batch(tasks)
    }

    /// Returns webview widget for the current view
    pub fn view(&self) -> Element<Action> {
        WebViewWidget::with(
            self.view_size,
            self.engine
                .get_view(self.get_current_view_id())
                .expect("failed to get view, because current view id does not exist"),
            self.engine
                .get_cursor(self.get_current_view_id())
                .expect("failed to get view, because current view id does not exist"),
        )
        .into()
    }
}

struct WebViewWidget {
    bounds: Size<u32>,
    image: Image<Handle>,
    cursor: Interaction,
}

impl WebViewWidget {
    fn with(bounds: Size<u32>, image: &ImageInfo, cursor: Interaction) -> Self {
        Self {
            bounds,
            image: image.as_image(),
            cursor,
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

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            self.cursor
        } else {
            mouse::Interaction::Idle
        }
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
