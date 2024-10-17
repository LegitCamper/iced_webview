use clipboard_rs::{Clipboard, ClipboardContext};
use iced::keyboard::{self};
use iced::mouse::{self, ScrollDelta};
use iced::{Point, Size};
use rand::Rng;
use smol_str::SmolStr;
use std::sync::{Arc, RwLock};
use ul_next::{
    config::Config,
    event::{self, KeyEventCreationInfo, MouseEvent, ScrollEvent},
    key_code::VirtualKeyCode,
    platform,
    renderer::Renderer,
    view,
    window::Cursor,
    Surface,
};
use url::Url;

use super::{Engine, PageType, PixelFormat};

struct UlClipboard {
    ctx: ClipboardContext,
}
impl platform::Clipboard for UlClipboard {
    fn clear(&mut self) {}

    fn read_plain_text(&mut self) -> Option<String> {
        Some(self.ctx.get_text().unwrap_or("".to_string()))
    }

    fn write_plain_text(&mut self, text: &str) {
        self.ctx
            .set_text(text.into())
            .expect("Failed to set contents of clipboard");
    }
}

type Views = Vec<View>;
struct View {
    id: usize,
    surface: Surface,
    view: view::View,
    cursor: Arc<RwLock<mouse::Interaction>>,
}

impl Into<super::View> for &View {
    fn into(self) -> super::View {
        super::View {
            title: self.view.title().unwrap(),
            url: self.view.url().unwrap(),
        }
    }
}

pub struct Ultralight {
    renderer: Renderer,
    view_config: view::ViewConfig,
    views: Views,
}

impl Default for Ultralight {
    fn default() -> Self {
        let config = Config::start().build().expect("Failed to start Ultralight");
        platform::enable_platform_fontloader();
        platform::enable_platform_filesystem(".").expect("Failed to access ultralight filesystem");
        platform::set_clipboard(UlClipboard {
            ctx: ClipboardContext::new().expect("Failed to get ownership of clipboard"),
        });

        let renderer = Renderer::create(config).expect("Failed to create ultralight renderer");
        let view_config = view::ViewConfig::start()
            .initial_device_scale(1.0)
            .font_family_standard("Arial")
            .is_accelerated(false)
            .build()
            .unwrap();

        Self {
            renderer,
            view_config,
            views: Views::new(),
        }
    }
}

impl Ultralight {
    pub fn new(font: &str, scale: f64, accelerated: bool) -> Self {
        accelerated.then(|| panic!("Ultralight acceleration is currently unsupported"));
        Self {
            view_config: view::ViewConfig::start()
                .initial_device_scale(scale)
                .font_family_standard(font)
                .is_accelerated(accelerated)
                .build()
                .unwrap(),
            ..Default::default()
        }
    }
}

impl Engine for Ultralight {
    fn do_work(&self) {
        self.renderer.update()
    }

    fn force_render(&self, id: usize) {
        self.views[id].view.set_needs_paint(true)
    }

    fn need_render(&self, id: usize) -> bool {
        self.views[id].view.needs_paint()
    }

    fn render(&mut self, _id: usize) {
        self.renderer.render();
    }

    fn resize(&mut self, size: Size<u32>) {
        self.views.iter().for_each(|view| {
            view.view.resize(size.width, size.height);
            view.surface.resize(size.width, size.height);
        })
    }

    fn pixel_buffer(&mut self, id: usize) -> Option<(PixelFormat, Vec<u8>)> {
        self.render(id);

        if let Some(pixel_data) = self.views[id].surface.lock_pixels() {
            let mut vec = Vec::new();
            vec.extend_from_slice(&pixel_data);
            Some((PixelFormat::Bgra, vec))
        } else {
            None
        }
    }

    fn get_cursor(&self, id: usize) -> mouse::Interaction {
        *self.views[id].cursor.read().unwrap()
    }

    fn goto_html(&self, id: usize, html: &str) {
        self.views[id].view.load_html(html).unwrap();
    }

    fn goto_url(&self, id: usize, url: &Url) {
        self.views[id].view.load_url(url.as_ref()).unwrap();
    }

    fn has_loaded(&self, id: usize) -> Option<bool> {
        Some(!self.views[id].view.is_loading())
    }

    fn get_views(&self) -> Vec<super::View> {
        self.views
            .iter()
            .map(|view| Into::<super::View>::into(view))
            .collect()
    }

    fn remove_view(&mut self, id: usize) {
        self.views.retain(|view| view.id != id)
    }

    fn get_view(&self, id: usize) -> super::View {
        Into::<super::View>::into(&self.views[id])
    }

    fn new_view(&mut self, page_type: PageType, size: Size<u32>) -> usize {
        let view = self
            .renderer
            .create_view(size.width, size.height, &self.view_config, None)
            .unwrap();

        let surface = view.surface().unwrap();
        match page_type {
            PageType::Url(url) => view.load_url(url).expect("Failed to load url"),
            PageType::Html(html) => view.load_html(html).expect("Failed to load custom html"),
        }

        // RGBA
        debug_assert!(surface.row_bytes() / size.width == 4);

        let cursor = Arc::new(RwLock::new(mouse::Interaction::Idle));
        let cb_cursor = cursor.clone();
        view.set_change_cursor_callback(move |_view, cursor_update| {
            *cb_cursor.write().unwrap() = match cursor_update {
                Cursor::None => mouse::Interaction::Idle,
                Cursor::Pointer => mouse::Interaction::Idle,
                Cursor::Hand => mouse::Interaction::Pointer,
                Cursor::Grab => mouse::Interaction::Grab,
                Cursor::VerticalText => mouse::Interaction::Text,
                Cursor::IBeam => mouse::Interaction::Text,
                Cursor::Cross => mouse::Interaction::Crosshair,
                Cursor::Wait => mouse::Interaction::Working,
                Cursor::Grabbing => mouse::Interaction::Grab,
                Cursor::NorthSouthResize => mouse::Interaction::ResizingVertically,
                Cursor::EastWestResize => mouse::Interaction::ResizingHorizontally,
                Cursor::NotAllowed => mouse::Interaction::NotAllowed,
                Cursor::ZoomIn => mouse::Interaction::ZoomIn,
                Cursor::ZoomOut => mouse::Interaction::ZoomIn,
                _ => mouse::Interaction::Pointer,
            };
        });

        let id = rand::thread_rng().gen();
        let view = View {
            id,
            surface,
            view,
            cursor,
        };

        self.views.push(view);
        id
    }

    fn refresh(&self, id: usize) {
        self.views[id].view.reload();
    }

    fn go_forward(&self, id: usize) {
        self.views[id].view.go_forward();
    }

    fn go_back(&self, id: usize) {
        self.views[id].view.go_back();
    }

    fn focus(&self, id: usize) {
        self.views[id].view.focus();
    }

    fn unfocus(&self, id: usize) {
        self.views[id].view.unfocus();
    }

    fn scroll(&self, id: usize, delta: ScrollDelta) {
        let scroll_event = match delta {
            ScrollDelta::Lines { x, y } => ScrollEvent::new(
                ul_next::event::ScrollEventType::ScrollByPixel,
                x as i32 * 100,
                y as i32 * 100,
            )
            .unwrap(),
            ScrollDelta::Pixels { x, y } => ScrollEvent::new(
                ul_next::event::ScrollEventType::ScrollByPixel,
                x as i32,
                y as i32,
            )
            .unwrap(),
        };
        self.views[id].view.fire_scroll_event(scroll_event);
    }

    fn handle_keyboard_event(&self, id: usize, event: keyboard::Event) {
        let key_event = match event {
            keyboard::Event::KeyPressed {
                key,
                location,
                modifiers,
                text,
                modified_key,
                physical_key: _,
            } => iced_key_to_ultralight_key(
                KeyPress::Press,
                Some(modified_key),
                Some(key),
                Some(location),
                modifiers,
                text,
            ),
            keyboard::Event::KeyReleased {
                key,
                location,
                modifiers,
            } => iced_key_to_ultralight_key(
                KeyPress::Unpress,
                None,
                Some(key),
                Some(location),
                modifiers,
                None,
            ),
            keyboard::Event::ModifiersChanged(modifiers) => {
                iced_key_to_ultralight_key(KeyPress::Press, None, None, None, modifiers, None)
            }
        };

        if let Some(key_event) = key_event {
            self.views[id].view.fire_key_event(key_event);
        }
    }

    fn handle_mouse_event(&mut self, id: usize, point: Point, event: mouse::Event) {
        match event {
            mouse::Event::ButtonReleased(mouse::Button::Forward) => {
                self.go_forward(id);
            }
            mouse::Event::ButtonReleased(mouse::Button::Back) => {
                self.go_back(id);
            }
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                self.views[id].view.fire_mouse_event(
                    MouseEvent::new(
                        ul_next::event::MouseEventType::MouseDown,
                        point.x as i32,
                        point.y as i32,
                        ul_next::event::MouseButton::Left,
                    )
                    .unwrap(),
                );
            }
            mouse::Event::ButtonReleased(mouse::Button::Left) => {
                self.views[id].view.fire_mouse_event(
                    MouseEvent::new(
                        ul_next::event::MouseEventType::MouseUp,
                        point.x as i32,
                        point.y as i32,
                        ul_next::event::MouseButton::Left,
                    )
                    .unwrap(),
                );
            }
            mouse::Event::ButtonPressed(mouse::Button::Right) => {
                self.views[id].view.fire_mouse_event(
                    MouseEvent::new(
                        ul_next::event::MouseEventType::MouseDown,
                        point.x as i32,
                        point.y as i32,
                        ul_next::event::MouseButton::Right,
                    )
                    .unwrap(),
                );
            }
            mouse::Event::ButtonReleased(mouse::Button::Right) => {
                self.views[id].view.fire_mouse_event(
                    MouseEvent::new(
                        ul_next::event::MouseEventType::MouseUp,
                        point.x as i32,
                        point.y as i32,
                        ul_next::event::MouseButton::Right,
                    )
                    .unwrap(),
                );
            }
            mouse::Event::CursorMoved { position: _ } => {
                self.views[id].view.fire_mouse_event(
                    MouseEvent::new(
                        ul_next::event::MouseEventType::MouseMoved,
                        point.x as i32,
                        point.y as i32,
                        ul_next::event::MouseButton::None,
                    )
                    .unwrap(),
                );
            }
            mouse::Event::WheelScrolled { delta } => self.scroll(id, delta),
            mouse::Event::CursorLeft => {
                self.unfocus(id);
            }
            mouse::Event::CursorEntered => {
                self.focus(id);
            }
            _ => (),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum KeyPress {
    Press,
    Unpress,
}

fn iced_key_to_ultralight_key(
    press: KeyPress,
    modified_key: Option<keyboard::Key>,
    key: Option<keyboard::Key>, // This one is modified by ctrl and results in wrong key
    _location: Option<keyboard::Location>,
    modifiers: keyboard::Modifiers,
    text: Option<SmolStr>,
) -> Option<event::KeyEvent> {
    let (text, virtual_key, native_key) = {
        if let Some(key) = key {
            let text = match key {
                keyboard::Key::Named(key) => {
                    if key == keyboard::key::Named::Space {
                        String::from(" ")
                    } else {
                        String::from("")
                    }
                }
                keyboard::Key::Character(_) => match text {
                    Some(text) => text.to_string(),
                    None => String::from(""),
                },
                keyboard::Key::Unidentified => return None,
            };
            let (virtual_key, native_key) = match key {
                keyboard::Key::Named(key) => match key {
                    keyboard::key::Named::Control => (
                        VirtualKeyCode::Control,
                        #[cfg(windows)]
                        17,
                        #[cfg(unix)]
                        29,
                    ),
                    keyboard::key::Named::Shift => (
                        VirtualKeyCode::Shift,
                        #[cfg(windows)]
                        16,
                        #[cfg(unix)]
                        42,
                    ),
                    keyboard::key::Named::Enter => (
                        VirtualKeyCode::Return,
                        #[cfg(windows)]
                        13,
                        #[cfg(unix)]
                        28,
                    ),
                    keyboard::key::Named::Tab => (
                        VirtualKeyCode::Tab,
                        #[cfg(windows)]
                        9,
                        #[cfg(unix)]
                        15,
                    ),
                    keyboard::key::Named::Space => (
                        VirtualKeyCode::Space,
                        #[cfg(windows)]
                        32,
                        #[cfg(unix)]
                        57,
                    ),
                    keyboard::key::Named::ArrowDown => (
                        VirtualKeyCode::Down,
                        #[cfg(windows)]
                        40,
                        #[cfg(unix)]
                        108,
                    ),
                    keyboard::key::Named::ArrowLeft => (
                        VirtualKeyCode::Right,
                        #[cfg(windows)]
                        37,
                        #[cfg(unix)]
                        106,
                    ),
                    keyboard::key::Named::ArrowRight => (
                        VirtualKeyCode::Up,
                        #[cfg(windows)]
                        39,
                        #[cfg(unix)]
                        103,
                    ),
                    keyboard::key::Named::ArrowUp => (
                        VirtualKeyCode::Left,
                        #[cfg(windows)]
                        33,
                        #[cfg(unix)]
                        105,
                    ),
                    keyboard::key::Named::End => (
                        VirtualKeyCode::End,
                        #[cfg(windows)]
                        35,
                        #[cfg(unix)]
                        107,
                    ),
                    keyboard::key::Named::Home => (
                        VirtualKeyCode::Home,
                        #[cfg(windows)]
                        36,
                        #[cfg(unix)]
                        102,
                    ),
                    keyboard::key::Named::Backspace => (
                        VirtualKeyCode::Back,
                        #[cfg(windows)]
                        8,
                        #[cfg(unix)]
                        14,
                    ),
                    keyboard::key::Named::Delete => (
                        VirtualKeyCode::Delete,
                        #[cfg(windows)]
                        46,
                        #[cfg(unix)]
                        11,
                    ),
                    keyboard::key::Named::Insert => (
                        VirtualKeyCode::Insert,
                        #[cfg(windows)]
                        45,
                        #[cfg(unix)]
                        110,
                    ),
                    keyboard::key::Named::Escape => (
                        VirtualKeyCode::Escape,
                        #[cfg(windows)]
                        27,
                        #[cfg(unix)]
                        1,
                    ),
                    keyboard::key::Named::F1 => (
                        VirtualKeyCode::F1,
                        #[cfg(windows)]
                        112,
                        #[cfg(unix)]
                        59,
                    ),
                    keyboard::key::Named::F2 => (
                        VirtualKeyCode::F2,
                        #[cfg(windows)]
                        113,
                        #[cfg(unix)]
                        60,
                    ),
                    keyboard::key::Named::F3 => (
                        VirtualKeyCode::F3,
                        #[cfg(windows)]
                        114,
                        #[cfg(unix)]
                        61,
                    ),
                    keyboard::key::Named::F4 => (
                        VirtualKeyCode::F4,
                        #[cfg(windows)]
                        115,
                        #[cfg(unix)]
                        62,
                    ),
                    keyboard::key::Named::F5 => (
                        VirtualKeyCode::F5,
                        #[cfg(windows)]
                        116,
                        #[cfg(unix)]
                        63,
                    ),
                    keyboard::key::Named::F6 => (
                        VirtualKeyCode::F6,
                        #[cfg(windows)]
                        117,
                        #[cfg(unix)]
                        64,
                    ),
                    keyboard::key::Named::F7 => (
                        VirtualKeyCode::F7,
                        #[cfg(windows)]
                        118,
                        #[cfg(unix)]
                        65,
                    ),
                    keyboard::key::Named::F8 => (
                        VirtualKeyCode::F8,
                        #[cfg(windows)]
                        119,
                        #[cfg(unix)]
                        66,
                    ),
                    keyboard::key::Named::F9 => (
                        VirtualKeyCode::F9,
                        #[cfg(windows)]
                        120,
                        #[cfg(unix)]
                        67,
                    ),
                    keyboard::key::Named::F10 => (
                        VirtualKeyCode::F10,
                        #[cfg(windows)]
                        121,
                        #[cfg(unix)]
                        68,
                    ),
                    keyboard::key::Named::F11 => (
                        VirtualKeyCode::F11,
                        #[cfg(windows)]
                        122,
                        #[cfg(unix)]
                        69,
                    ),
                    keyboard::key::Named::F12 => (
                        VirtualKeyCode::F12,
                        #[cfg(windows)]
                        123,
                        #[cfg(unix)]
                        70,
                    ),
                    _ => return None,
                },
                keyboard::Key::Character(key) => match key.as_str() {
                    "a" => (
                        VirtualKeyCode::A,
                        #[cfg(windows)]
                        65,
                        #[cfg(unix)]
                        30,
                    ),
                    "b" => (
                        VirtualKeyCode::B,
                        #[cfg(windows)]
                        66,
                        #[cfg(unix)]
                        48,
                    ),
                    "c" => (
                        VirtualKeyCode::C,
                        #[cfg(windows)]
                        67,
                        #[cfg(unix)]
                        46,
                    ),
                    "d" => (
                        VirtualKeyCode::D,
                        #[cfg(windows)]
                        68,
                        #[cfg(unix)]
                        32,
                    ),
                    "e" => (
                        VirtualKeyCode::E,
                        #[cfg(windows)]
                        69,
                        #[cfg(unix)]
                        18,
                    ),
                    "f" => (
                        VirtualKeyCode::F,
                        #[cfg(windows)]
                        70,
                        #[cfg(unix)]
                        33,
                    ),
                    "g" => (
                        VirtualKeyCode::G,
                        #[cfg(windows)]
                        71,
                        #[cfg(unix)]
                        34,
                    ),
                    "h" => (
                        VirtualKeyCode::H,
                        #[cfg(windows)]
                        72,
                        #[cfg(unix)]
                        35,
                    ),
                    "i" => (
                        VirtualKeyCode::I,
                        #[cfg(windows)]
                        73,
                        #[cfg(unix)]
                        23,
                    ),
                    "j" => (
                        VirtualKeyCode::J,
                        #[cfg(windows)]
                        74,
                        #[cfg(unix)]
                        36,
                    ),
                    "k" => (
                        VirtualKeyCode::K,
                        #[cfg(windows)]
                        75,
                        #[cfg(unix)]
                        37,
                    ),
                    "l" => (
                        VirtualKeyCode::L,
                        #[cfg(windows)]
                        76,
                        #[cfg(unix)]
                        38,
                    ),
                    "m" => (
                        VirtualKeyCode::M,
                        #[cfg(windows)]
                        77,
                        #[cfg(unix)]
                        50,
                    ),
                    "n" => (
                        VirtualKeyCode::N,
                        #[cfg(windows)]
                        78,
                        #[cfg(unix)]
                        49,
                    ),
                    "o" => (
                        VirtualKeyCode::O,
                        #[cfg(windows)]
                        79,
                        #[cfg(unix)]
                        24,
                    ),
                    "p" => (
                        VirtualKeyCode::P,
                        #[cfg(windows)]
                        80,
                        #[cfg(unix)]
                        25,
                    ),
                    "q" => (
                        VirtualKeyCode::Q,
                        #[cfg(windows)]
                        81,
                        #[cfg(unix)]
                        16,
                    ),
                    "r" => (
                        VirtualKeyCode::R,
                        #[cfg(windows)]
                        82,
                        #[cfg(unix)]
                        19,
                    ),
                    "s" => (
                        VirtualKeyCode::S,
                        #[cfg(windows)]
                        83,
                        #[cfg(unix)]
                        31,
                    ),
                    "t" => (
                        VirtualKeyCode::T,
                        #[cfg(windows)]
                        84,
                        #[cfg(unix)]
                        20,
                    ),
                    "u" => (
                        VirtualKeyCode::U,
                        #[cfg(windows)]
                        85,
                        #[cfg(unix)]
                        22,
                    ),
                    "v" => (
                        VirtualKeyCode::V,
                        #[cfg(windows)]
                        86,
                        #[cfg(unix)]
                        47,
                    ),
                    "w" => (
                        VirtualKeyCode::W,
                        #[cfg(windows)]
                        87,
                        #[cfg(unix)]
                        17,
                    ),
                    "x" => (
                        VirtualKeyCode::X,
                        #[cfg(windows)]
                        88,
                        #[cfg(unix)]
                        47,
                    ),
                    "y" => (
                        VirtualKeyCode::Y,
                        #[cfg(windows)]
                        89,
                        #[cfg(unix)]
                        21,
                    ),
                    "z" => (
                        VirtualKeyCode::Z,
                        #[cfg(windows)]
                        90,
                        #[cfg(unix)]
                        44,
                    ),
                    "0" => (
                        VirtualKeyCode::Key0,
                        #[cfg(windows)]
                        48,
                        #[cfg(unix)]
                        11,
                    ),
                    "1" => (
                        VirtualKeyCode::Key1,
                        #[cfg(windows)]
                        49,
                        #[cfg(unix)]
                        2,
                    ),
                    "2" => (
                        VirtualKeyCode::Key2,
                        #[cfg(windows)]
                        50,
                        #[cfg(unix)]
                        3,
                    ),
                    "3" => (
                        VirtualKeyCode::Key3,
                        #[cfg(windows)]
                        51,
                        #[cfg(unix)]
                        4,
                    ),
                    "4" => (
                        VirtualKeyCode::Key4,
                        #[cfg(windows)]
                        52,
                        #[cfg(unix)]
                        5,
                    ),
                    "5" => (
                        VirtualKeyCode::Key5,
                        #[cfg(windows)]
                        53,
                        #[cfg(unix)]
                        6,
                    ),
                    "6" => (
                        VirtualKeyCode::Key6,
                        #[cfg(windows)]
                        54,
                        #[cfg(unix)]
                        7,
                    ),
                    "7" => (
                        VirtualKeyCode::Key7,
                        #[cfg(windows)]
                        55,
                        #[cfg(unix)]
                        8,
                    ),
                    "8" => (
                        VirtualKeyCode::Key8,
                        #[cfg(windows)]
                        56,
                        #[cfg(unix)]
                        9,
                    ),
                    "9" => (
                        VirtualKeyCode::Key9,
                        #[cfg(windows)]
                        57,
                        #[cfg(unix)]
                        10,
                    ),
                    "," => (
                        VirtualKeyCode::OemComma,
                        #[cfg(windows)]
                        188,
                        #[cfg(unix)]
                        51,
                    ),
                    "." => (
                        VirtualKeyCode::OemPeriod,
                        #[cfg(windows)]
                        190,
                        #[cfg(unix)]
                        52,
                    ),
                    ";" => (
                        VirtualKeyCode::OemPeriod,
                        #[cfg(windows)]
                        186,
                        #[cfg(unix)]
                        39,
                    ),
                    "-" => (
                        VirtualKeyCode::OemMinus,
                        #[cfg(windows)]
                        189,
                        #[cfg(unix)]
                        12,
                    ),
                    "_" => (
                        VirtualKeyCode::OemMinus,
                        #[cfg(windows)]
                        189,
                        #[cfg(unix)]
                        74,
                    ),
                    "+" => (
                        VirtualKeyCode::OemPlus,
                        #[cfg(windows)]
                        187,
                        #[cfg(unix)]
                        78,
                    ),
                    "=" => (
                        VirtualKeyCode::OemPlus,
                        #[cfg(windows)]
                        187,
                        #[cfg(unix)]
                        78,
                    ),
                    "\\" => (
                        VirtualKeyCode::Oem5,
                        #[cfg(windows)]
                        220,
                        #[cfg(unix)]
                        43,
                    ),
                    "|" => (
                        VirtualKeyCode::Oem5,
                        #[cfg(windows)]
                        220,
                        #[cfg(unix)]
                        43,
                    ),
                    "`" => (
                        VirtualKeyCode::Oem3,
                        #[cfg(windows)]
                        192,
                        #[cfg(unix)]
                        41,
                    ),
                    "?" => (
                        VirtualKeyCode::Oem2,
                        #[cfg(windows)]
                        191,
                        #[cfg(unix)]
                        53,
                    ),
                    "/" => (
                        VirtualKeyCode::Oem2,
                        #[cfg(windows)]
                        191,
                        #[cfg(unix)]
                        53,
                    ),
                    ">" => (
                        VirtualKeyCode::Oem102,
                        #[cfg(windows)]
                        226,
                        #[cfg(unix)]
                        52,
                    ),
                    "<" => (
                        VirtualKeyCode::Oem102,
                        #[cfg(windows)]
                        226,
                        #[cfg(unix)]
                        52,
                    ),
                    "[" => (
                        VirtualKeyCode::Oem4,
                        #[cfg(windows)]
                        219,
                        #[cfg(unix)]
                        26,
                    ),
                    "]" => (
                        VirtualKeyCode::Oem6,
                        #[cfg(windows)]
                        221,
                        #[cfg(unix)]
                        27,
                    ),
                    _ => return None,
                },
                keyboard::Key::Unidentified => return None,
            };
            (text, virtual_key, native_key)
        } else {
            return None;
        }
    };

    let modifiers = event::KeyEventModifiers {
        alt: modifiers.alt(),
        ctrl: modifiers.control(),
        meta: modifiers.logo(),
        shift: modifiers.shift(),
    };

    let ty = if modifiers.ctrl {
        event::KeyEventType::RawKeyDown
    } else if !text.is_empty() && text.is_ascii() && press == KeyPress::Press {
        event::KeyEventType::Char
    } else {
        match press {
            KeyPress::Press => event::KeyEventType::RawKeyDown,
            KeyPress::Unpress => event::KeyEventType::KeyUp,
        }
    };

    let creation_info = KeyEventCreationInfo {
        ty,
        modifiers,
        virtual_key_code: virtual_key,
        native_key_code: native_key,
        text: text.as_str(),
        unmodified_text: if let Some(keyboard::Key::Character(char)) = modified_key {
            &char.to_string()
        } else {
            text.as_str()
        },
        is_keypad: false,
        is_auto_repeat: false,
        is_system_key: false,
    };

    event::KeyEvent::new(creation_info).ok()
}
