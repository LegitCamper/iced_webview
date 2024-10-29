use iced::{
    time,
    widget::{button, column, container, row, text},
    Element, Length, Subscription, Task,
};
use iced_webview::{Action, PageType, Ultralight, WebView};
use std::time::Duration;

static URL: &'static str = "https://docs.rs/iced/latest/iced/index.html";

fn main() -> iced::Result {
    iced::application("An embedded web view", App::update, App::view)
        .subscription(App::subscription)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    WebView(Action),
    ToggleWebviewVisibility,
    UrlChanged(String),
    NewWebViewCreated,
    CreateWebview,
    SwitchWebview,
}

struct App {
    webview: WebView<Ultralight, Message>,
    show_webview: bool,
    webview_url: Option<String>,
    num_views: u32,
    current_view: Option<u32>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let webview = WebView::new()
            .on_create_view(Message::NewWebViewCreated)
            .on_url_change(Message::UrlChanged);
        (
            Self {
                webview,
                show_webview: false,
                webview_url: None,
                num_views: 0,
                current_view: None,
            },
            // Create the first webview so its available once toggled
            Task::done(Message::CreateWebview),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewWebViewCreated => {
                if self.current_view == None {
                    self.current_view = Some(0);
                    // if its the first tab change to it, after that require switching manually
                    return Task::done(Action::ChangeView(0)).map(Message::WebView);
                }
                self.num_views += 1;
                Task::none()
            }
            Message::WebView(msg) => self.webview.update(msg),
            Message::ToggleWebviewVisibility => {
                self.show_webview = !self.show_webview;
                Task::none()
            }
            Message::UrlChanged(new_url) => {
                self.webview_url = Some(new_url);
                Task::none()
            }
            Message::CreateWebview => self
                .webview
                .update(Action::CreateView(PageType::Url(URL.to_string()))),
            Message::SwitchWebview => {
                if let Some(current_view) = self.current_view.as_mut() {
                    if *current_view + 1 > self.num_views {
                        *current_view = 0;
                    } else {
                        *current_view += 1;
                    };
                    self.webview.update(Action::ChangeView(*current_view))
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let mut column = column![row![
            text(if !self.show_webview {
                "Click the button to open a webview"
            } else {
                "Iced docs can be pulled up inside an iced app?! Whoa!"
            }),
            container(row![
                button("Toggle web view(s)").on_press(Message::ToggleWebviewVisibility),
                button("New web view").on_press(Message::CreateWebview),
                button("Switch views").on_press(Message::SwitchWebview),
            ])
            .align_right(Length::Fill)
        ]];

        if self.show_webview {
            if let Some(current_view) = self.current_view {
                column = column.push(column![
                    text(format!("view index: {}", current_view)),
                    self.webview.view().map(Message::WebView),
                    text(format!("Url: {:?}", self.webview_url)),
                ]);
            }
        }
        column.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(10))
            .map(|_| Action::Update)
            .map(Message::WebView)
    }
}
