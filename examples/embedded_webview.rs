use iced::{
    time,
    widget::{button, column, container, row, text},
    Element, Length, Subscription, Task,
};
use iced_webview::{webview, PageType, Ultralight, WebView};
use std::time::Duration;

fn main() -> iced::Result {
    iced::application("An embedded web view", App::update, App::view)
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
    num_tabs: u32,
    current_tab: u32,
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
                num_tabs: 1,
                current_tab: 0,
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
                self.num_tabs += 1;
                self.webview.update(webview::Action::CreateTab)
            }
            Message::SwitchWebview => {
                if self.current_tab + 1 >= self.num_tabs {
                    self.current_tab = 0;
                } else {
                    self.current_tab += 1;
                };
                let tab = iced_webview::TabSelectionType::Index(self.current_tab as usize);
                self.webview.update(webview::Action::ChangeTab(tab))
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
                text(format!("view index: {}", self.current_tab)),
                self.webview.view().map(Message::WebView),
                text(format!("Url: {:?}", self.webview_url)),
            ])
        } else {
            None
        })
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(10))
            .map(|_| webview::Action::Update)
            .map(Message::WebView)
    }
}