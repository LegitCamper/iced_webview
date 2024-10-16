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
    CloseView(usize),
    CreateView,
    GoBackward(usize),
    GoForward(usize),
    GoToUrl(usize, Url),
    Refresh(usize),
    SendKeyboardEvent(usize, keyboard::Event),
    SendMouseEvent(usize, mouse::Event, Point),
    Update,
    Resize(Rectangle),
}

pub struct WebView<Engine, Message>
where
    Engine: engines::Engine,
{
    engine: Engine,
    view_size: Rectangle,
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
                view_size: Rectangle {
                    x: 0.,
                    y: 0.,
                    width: ImageInfo::WIDTH,
                    height: ImageInfo::HEIGHT,
                },
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

    fn update_engine(&mut self, id: usize) {
        self.engine.do_work();
        if self.engine.has_loaded(id) {
            if self.engine.need_render(id) {
                let (format, image_data) = self.engine.pixel_buffer(id, self.view_size);
                let view = ImageInfo::new(image_data, format, self.view_size);
                self.engine
                    .get_views_mut()
                    .get_mut(id)
                    .expect(Engine::CANNOT_FIND_VIEW)
                    .set_view(view)
            }
        } else {
            let view = ImageInfo {
                width: self.view_size.width,
                height: self.view_size.height,
                ..Default::default()
            };
            self.engine
                .get_views_mut()
                .get_mut(id)
                .expect(Engine::CANNOT_FIND_VIEW)
                .set_view(view)
        }
    }

    fn force_update(&mut self, id: usize) {
        self.engine.do_work();
        let (format, image_data) = self.engine.pixel_buffer(id, self.view_size);
        let current_view = self
            .engine
            .get_views_mut()
            .get_mut(id)
            .expect(Engine::CANNOT_FIND_VIEW);
        let view = ImageInfo::new(image_data, format, self.view_size);
        current_view.set_view(view);
    }

    pub fn update(&mut self, action: Action) -> Task<Message> {
        let mut tasks = Vec::new();
        let ids: Vec<usize> = self
            .engine
            .get_views()
            .iter()
            .map(|view| view.id())
            .collect();
        for id in ids {
            self.update_engine(id);
            if let Some(on_url_change) = &self.on_url_change {
                if let Some(current_view) = self.engine.get_views().get(id) {
                    if self.url != current_view.url() {
                        self.url = current_view.url();
                        tasks.push(Task::done(on_url_change(self.url.clone())))
                    }
                }
            }
            if let Some(on_title_change) = &self.on_title_change {
                if let Some(current_view) = self.engine.get_views().get(id) {
                    if self.title != current_view.title() {
                        self.title = current_view.title();
                        tasks.push(Task::done(on_title_change(self.title.clone())))
                    }
                }
            }
        }
        tasks.push(match action {
            Action::CloseView(id) => {
                self.engine.get_views_mut().remove(id);
                if let Some(on_close_view) = &self.on_close_view {
                    Task::done((on_close_view)(id as usize))
                } else {
                    Task::none()
                }
            }
            Action::CreateView => {
                let bounds = self.view_size;
                let id = self.engine.new_view(
                    self.new_view.clone(),
                    Rectangle {
                        x: 0.,
                        y: 0.,
                        // Without this new views are broken ?? idek
                        width: bounds.width + 10.,
                        height: bounds.height - 10.,
                    },
                );
                match self.new_view {
                    PageType::Url(url) => self
                        .engine
                        .goto_url(id, &Url::parse(url).expect("Failed to parse new view url")),
                    PageType::Html(html) => self.engine.goto_html(id, html),
                }
                self.engine.force_need_render(id);
                if let Some(on_create_view) = &self.on_create_view {
                    Task::done((on_create_view)(id))
                } else {
                    Task::none()
                }
            }
            Action::GoBackward(id) => {
                self.engine.go_back(id);
                Task::none()
            }
            Action::GoForward(id) => {
                self.engine.go_forward(id);
                Task::none()
            }
            Action::GoToUrl(id, url) => {
                self.engine.goto_url(id, &url);
                Task::none()
            }
            Action::Refresh(id) => {
                self.engine.refresh(id);
                Task::none()
            }
            Action::SendKeyboardEvent(id, event) => {
                self.engine.handle_keyboard_event(id, event);
                Task::none()
            }
            Action::SendMouseEvent(id, point, event) => {
                self.engine.handle_mouse_event(id, event, point);
                Task::none()
            }
            Action::Update => {
                let ids: Vec<usize> = self
                    .engine
                    .get_views()
                    .iter()
                    .map(|view| view.id())
                    .collect();
                for id in ids {
                    self.force_update(id);
                }
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

    pub fn view(&self, id: usize) -> Element<Action> {
        if let Some(current_view) = self.engine.get_views().get(id) {
            WebViewWidget::new(id, self.view_size, current_view.get_view())
        } else {
            WebViewWidget::new(id, self.view_size, &ImageInfo::default())
        }
        .into()
    }
}

pub struct WebViewWidget {
    id: usize,
    bounds: Rectangle,
    image: Image<Handle>,
}

impl WebViewWidget {
    pub fn new(id: usize, bounds: Rectangle, image: &ImageInfo) -> Self {
        Self {
            id,
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
        if self.bounds != layout.bounds() {
            shell.publish(Action::Resize(layout.bounds()));
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
