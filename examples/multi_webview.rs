use iced::{
    time,
    widget::{column, container, row, text},
    Element, Length, Subscription, Task,
};
use iced_webview::{
    advanced::{Action, WebView},
    PageType, Ultralight, ViewId,
};
use std::time::Duration;

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
    webviews: (Option<ViewId>, Option<ViewId>),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let webview = WebView::new().on_create_view(Message::CreatedNewWebView);
        (
            Self {
                webview,
                webviews: (None, None),
            },
            Task::chain(
                Task::done(Action::CreateView(PageType::Url(URL1.to_string())))
                    .map(Message::WebView),
                Task::done(Action::CreateView(PageType::Url(URL2.to_string())))
                    .map(Message::WebView),
            ),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WebView(msg) => self.webview.update(msg),
            Message::CreatedNewWebView(view_id) => {
                if self.webviews.0 == None {
                    self.webviews.0 = Some(view_id);
                } else if self.webviews.1 == None {
                    self.webviews.1 = Some(view_id);
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let Some(view1) = self.webviews.0 else {
            return text("loading").into();
        };
        let Some(view2) = self.webviews.1 else {
            return text("loading").into();
        };
        row![
            container(column![
                text("View 1 of iced docs"),
                container(self.webview.view(view1).map(Message::WebView)).height(Length::Fill),
            ])
            .padding(5),
            container(column![
                text("View 2 of the iced_webview repo"),
                container(self.webview.view(view2).map(Message::WebView)).height(Length::Fill),
            ])
            .padding(5),
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(10))
            .map(|_| Action::Update)
            .map(Message::WebView)
    }
}
