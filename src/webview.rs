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

use crate::{engines, ImageInfo, PageType};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ChangeView(usize),
    CloseCurrentView,
    CloseView(usize),
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
    view_size: Size<u32>,
    new_view: PageType,
    on_close_view: Option<Box<dyn Fn(usize) -> Message>>,
    on_create_view: Option<Box<dyn Fn(usize) -> Message>>,
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

    pub fn on_create_view(mut self, on_create_view: impl Fn(usize) -> Message + 'static) -> Self {
        self.on_create_view = Some(Box::new(on_create_view));
        self
    }

    pub fn on_close_view(mut self, on_close_view: impl Fn(usize) -> Message + 'static) -> Self {
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
        if let Some(has_loaded) = self.engine.has_loaded() {
            if has_loaded {
                if self.engine.need_render() {
                    if let Some((format, image_data)) = self.engine.pixel_buffer() {
                        let view = ImageInfo::new(
                            image_data,
                            format,
                            self.view_size.width,
                            self.view_size.height,
                        );
                        self.engine
                            .get_views_mut()
                            .get_current_mut()
                            .expect("Unable to get current view id")
                            .set_view(view)
                    }
                }
            } else {
                let view = ImageInfo {
                    width: self.view_size.width,
                    height: self.view_size.height,
                    ..Default::default()
                };
                self.engine
                    .get_views_mut()
                    .get_current_mut()
                    .expect("Unable to get current view id")
                    .set_view(view)
            }
        }
    }

    fn force_update(&mut self) {
        self.engine.do_work();
        if let Some((format, image_data)) = self.engine.pixel_buffer() {
            if let Some(current_view) = self.engine.get_views_mut().get_current_mut() {
                let view = ImageInfo::new(
                    image_data,
                    format,
                    self.view_size.width,
                    self.view_size.height,
                );
                current_view.set_view(view);
            }
        }
    }

    pub fn update(&mut self, action: Action) -> Task<Message> {
        self.update_engine();
        let mut tasks = Vec::new();
        if let Some(on_url_change) = &self.on_url_change {
            if let Some(current_view) = self.engine.get_views().get_current() {
                if self.url != current_view.url() {
                    self.url = current_view.url();
                    tasks.push(Task::done(on_url_change(self.url.clone())))
                }
            }
        }
        if let Some(on_title_change) = &self.on_title_change {
            if let Some(current_view) = self.engine.get_views().get_current() {
                if self.title != current_view.title() {
                    self.title = current_view.title();
                    tasks.push(Task::done(on_title_change(self.title.clone())))
                }
            }
        }
        tasks.push(match action {
            Action::ChangeView(id) => {
                self.engine.get_views_mut().set_current_id(id);
                Task::none()
            }
            Action::CloseCurrentView => {
                let id = self
                    .engine
                    .get_views()
                    .get_current_id()
                    .expect("Unable to get current view id");

                self.engine.get_views_mut().remove(id);

                if let Some(on_view_close) = &self.on_close_view {
                    Task::done((on_view_close)(id))
                } else {
                    Task::none()
                }
            }
            Action::CloseView(id) => {
                self.engine.get_views_mut().remove(id);

                if let Some(on_view_close) = &self.on_close_view {
                    Task::done((on_view_close)(id))
                } else {
                    Task::none()
                }
            }
            Action::CreateView => {
                let new_view = self.new_view.clone();
                let bounds = self.view_size;
                let view = self.engine.new_view(
                    new_view.clone(),
                    Size::new(bounds.width + 10, bounds.height - 10),
                );
                let id = self.engine.get_views_mut().insert(view);
                self.engine.get_views_mut().set_current_id(id);
                self.engine.force_need_render();
                self.engine.resize(bounds);
                match new_view {
                    PageType::Url(url) => self
                        .engine
                        .goto_url(&Url::parse(url).expect("Failed to parse new view url")),
                    PageType::Html(html) => self.engine.goto_html(html),
                }
                Task::none()
            }
            Action::GoBackward => {
                self.engine.go_back();
                Task::none()
            }
            Action::GoForward => {
                self.engine.go_forward();
                Task::none()
            }
            Action::GoToUrl(url) => {
                self.engine.goto_url(&url);
                Task::none()
            }
            Action::Refresh => {
                self.engine.refresh();
                Task::none()
            }
            Action::SendKeyboardEvent(event) => {
                self.engine.handle_keyboard_event(event);
                Task::none()
            }
            Action::SendMouseEvent(point, event) => {
                self.engine.handle_mouse_event(event, point);
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
        if let Some(current_view) = self.engine.get_views().get_current() {
            WebViewWidget::new(self.view_size, current_view.get_view()).into()
        } else {
            WebViewWidget::new(self.view_size, &ImageInfo::default()).into()
        }
    }

    pub fn view_id(&self, id: usize) -> Element<Action> {
        if let Some(current_view) = self.engine.get_views().get(id) {
            WebViewWidget::new(self.view_size, current_view.get_view()).into()
        } else {
            WebViewWidget::new(self.view_size, &ImageInfo::default()).into()
        }
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
