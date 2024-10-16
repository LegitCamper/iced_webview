use iced::{
    time,
    widget::{button, column, container, row, text},
    Element, Length, Subscription, Task,
};
use iced_webview::{webview, Action, PageType, Ultralight, WebView};
use std::time::Duration;

fn main() -> iced::Result {
    iced::application("An embedded web view", App::update, App::view)
        .antialiasing(true)
        .subscription(App::subscription)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    WebView(webview::Action),
    ToggleWebviewVisibility,
    UpdateWebviewTitle(String),
    CreateWebview,
    SwitchWebview,
}

struct App {
    webview: WebView<Ultralight, Message>,
    show_webview: bool,
    webview_url: Option<String>,
    num_views: usize,
    view_ids: Vec<usize>,
    current_view: usize,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let page = PageType::Url("https://docs.rs/iced/latest/iced/index.html");
        let (mut webview, task) = WebView::new(page);
        webview = webview.on_url_change(Message::UpdateWebviewTitle);
        (
            Self {
                webview,
                show_webview: false,
                webview_url: None,
                num_views: 1,
                view_ids: Vec::new(),
                current_view: 0,
            },
            task.map(Message::WebView),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
                if self.current_view + 1 >= self.num_views {
                    self.current_view = 0;
                } else {
                    self.current_view += 1;
                };
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![row![
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
        ]]
        .push_maybe(if self.show_webview {
            Some(column![
                text(format!("view index: {}", self.current_view)),
                self.webview
                    .view(self.view_ids[self.current_view])
                    .map(Message::WebView),
                text(format!("Url: {:?}", self.webview_url)),
            ])
        } else {
            None
        })
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(10))
            .map(|_| Action::Update)
            .map(Message::WebView)
    }
}
