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
    engine: Engine,
    current_view: Option<ViewId>,
    view: ImageInfo,
    view_size: Size<u32>,
    new_view: PageType,
    on_close_view: Option<Box<dyn Fn(ViewId) -> Message>>,
    on_create_view: Option<Box<dyn Fn(ViewId) -> Message>>,
    on_url_change: Option<Box<dyn Fn(String) -> Message>>,
    url: String,
    on_title_change: Option<Box<dyn Fn(String) -> Message>>,
    title: String,
}

impl<Engine: engines::Engine + Default, Message: Send + Clone + 'static> WebView<Engine, Message> {
    pub fn new(new_view: PageType) -> (Self, Task<Action>) {
        (
            WebView {
                engine: Engine::default(),
                current_view: None,
                view: ImageInfo::default(),
                view_size: Size::new(1920, 1080),
                new_view,
                on_close_view: None,
                on_create_view: None,
                on_url_change: None,
                url: String::new(),
                on_title_change: None,
                title: String::new(),
            },
            Task::done(Action::CreateView),
        )
    }

    pub fn on_create_view(mut self, on_create_view: impl Fn(ViewId) -> Message + 'static) -> Self {
        self.on_create_view = Some(Box::new(on_create_view));
        self
    }

    pub fn on_close_view(mut self, on_close_view: impl Fn(ViewId) -> Message + 'static) -> Self {
        self.on_close_view = Some(Box::new(on_close_view));
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

    fn update_engine(&mut self) {
        self.engine.do_work();
        if let Some(current_view) = self.current_view {
            if let Some(has_loaded) = self.engine.has_loaded(current_view) {
                if has_loaded {
                    if self.engine.need_render(current_view) {
                        if let Some((format, image_data)) = self.engine.pixel_buffer(current_view) {
                            self.view = ImageInfo::new(
                                image_data,
                                format,
                                self.view_size.width,
                                self.view_size.height,
                            );
                            return;
                        }
                    }
                }
            }
        }
        self.view = ImageInfo {
            width: self.view_size.width,
            height: self.view_size.height,
            ..Default::default()
        };
    }

    fn force_update(&mut self) {
        self.engine.do_work();
        if let Some(current_view) = self.current_view {
            if let Some((format, image_data)) = self.engine.pixel_buffer(current_view) {
                self.view = ImageInfo::new(
                    image_data,
                    format,
                    self.view_size.width,
                    self.view_size.height,
                );
            }
        }
    }

    pub fn update(&mut self, action: Action) -> Task<Message> {
        self.update_engine();
        let mut tasks = Vec::new();
        if let Some(current_view) = self.current_view {
            if let Some(on_url_change) = &self.on_url_change {
                let current_view = self.engine.get_view(current_view);
                if self.url != current_view.url {
                    self.url = current_view.url;
                    tasks.push(Task::done(on_url_change(self.url.clone())))
                }
            }
            if let Some(on_title_change) = &self.on_title_change {
                let current_view = self.engine.get_view(current_view);
                if self.title != current_view.title {
                    self.title = current_view.title;
                    tasks.push(Task::done(on_title_change(self.title.clone())))
                }
            }
        }
        tasks.push(match action {
            Action::ChangeView(id) => {
                self.current_view = Some(id);
                Task::none()
            }
            Action::CloseView(id) => {
                self.engine.remove_view(id);

                if let Some(on_view_close) = &self.on_close_view {
                    Task::done((on_view_close)(id))
                } else {
                    Task::none()
                }
            }
            Action::CreateView => {
                let bounds = self.view_size;
                let view_id = self.engine.new_view(
                    self.new_view.clone(),
                    Size::new(bounds.width + 10, bounds.height - 10), // ??? fixes wonky image
                );
                match self.new_view {
                    PageType::Url(url) => self.engine.goto_url(
                        view_id,
                        &Url::parse(url).expect("Failed to parse new view url"),
                    ),
                    PageType::Html(html) => self.engine.goto_html(view_id, html),
                }
                self.engine.force_render(view_id);
                if let Some(on_create_view) = &self.on_create_view {
                    Task::done((on_create_view)(view_id))
                } else {
                    Task::none()
                }
            }
            Action::GoBackward => {
                if let Some(current_view) = self.current_view {
                    self.engine.go_back(current_view);
                }
                Task::none()
            }
            Action::GoForward => {
                if let Some(current_view) = self.current_view {
                    self.engine.go_forward(current_view);
                }
                Task::none()
            }
            Action::GoToUrl(url) => {
                if let Some(current_view) = self.current_view {
                    self.engine.goto_url(current_view, &url);
                }
                Task::none()
            }
            Action::Refresh => {
                if let Some(current_view) = self.current_view {
                    self.engine.refresh(current_view);
                }
                Task::none()
            }
            Action::SendKeyboardEvent(event) => {
                if let Some(current_view) = self.current_view {
                    self.engine.handle_keyboard_event(current_view, event);
                }
                Task::none()
            }
            Action::SendMouseEvent(point, event) => {
                if let Some(current_view) = self.current_view {
                    self.engine.handle_mouse_event(current_view, event, point);
                }
                Task::none()
            }
            Action::Update => {
                self.force_update();
                Task::none()
            }
            Action::Resize(size) => {
                self.view_size = size;
                self.engine.resize(size);
                Task::none()
            }
        });

        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    pub fn view(&self) -> Element<Action> {
        WebViewWidget::new(self.view_size, &self.view).into()
    }
}

pub struct WebViewWidget {
    bounds: Size<u32>,
    image: Image<Handle>,
}

impl WebViewWidget {
    pub fn new(bounds: Size<u32>, image: &ImageInfo) -> Self {
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
