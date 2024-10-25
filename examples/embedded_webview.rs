use iced::{
    time,
    widget::{button, column, container, row, text},
    Element, Length, Subscription, Task,
};
use iced_webview::{Action, Ultralight, ViewId, WebView};
use std::time::Duration;
use url::Url;

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
    UpdateWebviewTitle(String),
    CreatedNewWebView,
    CreateWebview,
    SwitchWebview,
}

struct App {
    webview: WebView<Ultralight, Message>,
    show_webview: bool,
    webview_url: Option<String>,
    num_views: u32,
    current_view: Option<ViewId>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let webview = WebView::new()
            // This is what allows us go to a new url
            .on_create_view(Message::CreatedNewWebView)
            .on_url_change(Message::UpdateWebviewTitle);
        (
            Self {
                webview,
                show_webview: false,
                webview_url: None,
                num_views: 1,
                current_view: None,
            },
            // Create the first webview so its available once toggled
            Task::done(Message::CreateWebview),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CreatedNewWebView => {
                // Now that we know the webview has been created lets navigate to URL
                let url = Url::parse(URL).unwrap();
                Task::done(Action::GoToUrl(url)).map(Message::WebView)
            }
            Message::WebView(msg) => self.webview.update(msg),
            Message::ToggleWebviewVisibility => {
                self.show_webview = !self.show_webview;
                Task::none()
            }
            Message::UpdateWebviewTitle(new_title) => {
                self.webview_url = Some(new_title);
                Task::none()
            }
            Message::CreateWebview => {
                self.num_views += 1;
                self.webview.update(Action::CreateView)
            }
            Message::SwitchWebview => {
                if let Some(current_view) = self.current_view.as_mut() {
                    if *current_view + 1 >= self.num_views as ViewId {
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
