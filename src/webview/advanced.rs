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
use engines::EngineResult;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    CloseView(ViewId),
    CreateView(PageType),
    GoBackward(ViewId),
    GoForward(ViewId),
    GoToUrl(ViewId, Url),
    Refresh(ViewId),
    SendKeyboardEvent(ViewId, keyboard::Event),
    SendMouseEvent(ViewId, mouse::Event, Point),
    Update,
    Resize(Size<u32>),
}

pub struct WebView<Engine, Message>
where
    Engine: engines::Engine,
{
    engine: Engine,
    view_size: Size<u32>,
    on_close_view: Option<Box<dyn Fn(ViewId) -> Message>>,
    on_create_view: Option<Box<dyn Fn(ViewId) -> Message>>,
    on_url_change: Option<Box<dyn Fn(ViewId, String) -> Message>>,
    urls: Vec<(ViewId, String)>,
    on_title_change: Option<Box<dyn Fn(ViewId, String) -> Message>>,
    titles: Vec<(ViewId, String)>,
}

impl<Engine: engines::Engine + Default, Message: Send + Clone + 'static> Default
    for WebView<Engine, Message>
{
    fn default() -> Self {
        WebView {
            engine: Engine::default(),
            view_size: Size::new(1920, 1080),
            on_close_view: None,
            on_create_view: None,
            on_url_change: None,
            urls: Vec::new(),
            on_title_change: None,
            titles: Vec::new(),
        }
    }
}

impl<Engine: engines::Engine + Default, Message: Send + Clone + 'static> WebView<Engine, Message> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_create_view(mut self, on_create_view: impl Fn(usize) -> Message + 'static) -> Self {
        self.on_create_view = Some(Box::new(on_create_view));
        self
    }

    pub fn on_close_view(mut self, on_close_view: impl Fn(usize) -> Message + 'static) -> Self {
        self.on_close_view = Some(Box::new(on_close_view));
        self
    }

    pub fn on_url_change(
        mut self,
        on_url_change: impl Fn(ViewId, String) -> Message + 'static,
    ) -> Self {
        self.on_url_change = Some(Box::new(on_url_change));
        self
    }

    pub fn on_title_change(
        mut self,
        on_title_change: impl Fn(ViewId, String) -> Message + 'static,
    ) -> Self {
        self.on_title_change = Some(Box::new(on_title_change));
        self
    }

    /// Passes update to webview
    pub fn update(&mut self, action: Action) -> Task<Message> {
        let mut tasks = Vec::new();

        // Check url & title for changes and callback if so
        for (id, url) in self.urls.iter_mut() {
            if let Some(on_url_change) = &self.on_url_change {
                if let EngineResult::Success(engine_url) = self.engine.get_url(*id) {
                    if *url != engine_url {
                        *url = engine_url.clone();
                        tasks.push(Task::done(on_url_change(*id, engine_url)));
                    }
                }
            }
        }
        for (id, title) in self.titles.iter_mut() {
            if let Some(on_title_change) = &self.on_title_change {
                if let EngineResult::Success(engine_title) = self.engine.get_title(*id) {
                    if *title != engine_title {
                        *title = engine_title.clone();
                        tasks.push(Task::done(on_title_change(*id, engine_title)));
                    }
                }
            }
        }

        match action {
            Action::CloseView(id) => {
                self.engine.remove_view(id);
                self.urls.retain(|url| url.0 != id);
                self.titles.retain(|title| title.0 != id);

                if let Some(on_view_close) = &self.on_close_view {
                    tasks.push(Task::done((on_view_close)(id)))
                }
            }
            Action::CreateView(page_type) => {
                let id = self
                    .engine
                    .new_view(self.view_size)
                    .expect("Failed to create new view");
                self.urls.push((id, String::new()));
                self.titles.push((id, String::new()));
                self.engine
                    .goto(id, page_type)
                    .expect("Failed to page type");

                if let Some(on_view_create) = &self.on_create_view {
                    tasks.push(Task::done((on_view_create)(id)))
                }
            }
            Action::GoBackward(id) => {
                self.engine.go_back(id);
                self.engine.request_render(id, self.view_size);
            }
            Action::GoForward(id) => {
                self.engine.go_forward(id);
                self.engine.request_render(id, self.view_size);
            }
            Action::GoToUrl(id, url) => {
                self.engine.goto(id, PageType::Url(url.to_string()));
                self.engine.request_render(id, self.view_size);
            }
            Action::Refresh(id) => {
                self.engine.refresh(id);
                self.engine.request_render(id, self.view_size);
            }
            Action::SendKeyboardEvent(id, event) => {
                self.engine.handle_keyboard_event(id, event);
                self.engine.request_render(id, self.view_size);
            }
            Action::SendMouseEvent(id, point, event) => {
                self.engine.handle_mouse_event(id, event, point);
                self.engine.request_render(id, self.view_size);
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

        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    /// Like a normal `view()` method in iced, but takes an id of the desired view
    pub fn view(&self, id: usize) -> Element<Action> {
        let view = match self.engine.get_view(id) {
            EngineResult::IdDoesNotExist => panic!("Requested Id does not exist"),
            EngineResult::NotLoaded => {
                return WebViewWidget::new(
                    id,
                    self.view_size,
                    &ImageInfo::default(),
                    Interaction::None,
                )
                .into()
            }
            EngineResult::Success(view) => view,
        };
        let cursor = match self.engine.get_cursor(id) {
            EngineResult::IdDoesNotExist => panic!("Requested Id does not exist"),
            EngineResult::NotLoaded => {
                return WebViewWidget::new(
                    id,
                    self.view_size,
                    &ImageInfo::default(),
                    Interaction::None,
                )
                .into()
            }
            EngineResult::Success(cursor) => cursor,
        };
        WebViewWidget::new(id, self.view_size, view, cursor).into()
    }
}

struct WebViewWidget {
    id: ViewId,
    bounds: Size<u32>,
    image: Image<Handle>,
    cursor: Interaction,
}

impl WebViewWidget {
    fn new(id: ViewId, bounds: Size<u32>, image: &ImageInfo, cursor: Interaction) -> Self {
        Self {
            id,
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
                shell.publish(Action::SendKeyboardEvent(self.id, event));
            }
            Event::Mouse(event) => {
                if let Some(point) = cursor.position_in(layout.bounds()) {
                    shell.publish(Action::SendMouseEvent(self.id, event, point));
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
