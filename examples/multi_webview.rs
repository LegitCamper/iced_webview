use iced::{
    time,
    widget::{row, text},
    Element, Subscription, Task,
};
use iced_webview::{
    advanced::{Action, WebView},
    Ultralight, ViewId,
};
use std::time::Duration;
use url::Url;

static URL1: &'static str = "https://docs.rs/iced/latest/iced/index.html";
static URL2: &'static str = "https://github.com/LegitCamper/iced_webview";

fn main() -> iced::Result {
    iced::application("An multi webview application", App::update, App::view)
        .subscription(App::subscription)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    WebView(Action),
    CreatedNewWebView(ViewId),
}

struct App {
    webview: WebView<Ultralight, Message>,
    web_views: (Option<ViewId>, Option<ViewId>),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let webview = WebView::new().on_create_view(Message::CreatedNewWebView);
        (
            Self {
                webview,
                web_views: (None, None),
            },
            Task::chain(
                Task::done(Action::CreateView).map(Message::WebView),
                Task::done(Action::CreateView).map(Message::WebView),
            ),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WebView(msg) => self.webview.update(msg),
            Message::CreatedNewWebView(view_id) => {
                if self.web_views.0 == None {
                    self.web_views.0 = Some(view_id);
                    Task::done(Action::GoToUrl(view_id, Url::parse(URL1).unwrap()))
                        .map(Message::WebView)
                } else {
                    self.web_views.1 = Some(view_id);
                    Task::done(Action::GoToUrl(view_id, Url::parse(URL2).unwrap()))
                        .map(Message::WebView)
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let Some(view1) = self.web_views.0 else {
            return text("loading").into();
        };
        let Some(view2) = self.web_views.1 else {
            return text("loading").into();
        };
        row![
            self.webview.view(view1).map(Message::WebView),
            self.webview.view(view2).map(Message::WebView)
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(10))
            .map(|_| Action::Update)
            .map(Message::WebView)
    }
}