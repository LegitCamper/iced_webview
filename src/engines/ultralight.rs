use clipboard_rs::{Clipboard, ClipboardContext};
use iced::keyboard::{self};
use iced::mouse::{self, ScrollDelta};
use iced::{Point, Size};
use rand::Rng;
use smol_str::SmolStr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::{env::var, path::Path};
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

use super::{Engine, EngineResult, PageType, PixelFormat, ViewId};
use crate::ImageInfo;

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

/// Holds Ultralight View info like surfaces for rendering and urls & titles
pub struct View {
    id: ViewId,
    surface: Surface,
    view: view::View,
    cursor: Arc<RwLock<mouse::Interaction>>,
    last_frame: ImageInfo,
}

/// Implementation of the Ultralight browsing engine for iced_webivew
pub struct Ultralight {
    renderer: Renderer,
    view_config: view::ViewConfig,
    views: Vec<View>,
}

impl Default for Ultralight {
    /// Creates a default CPU rendered ultralight config
    fn default() -> Self {
        let config = Config::start()
            .build()
            .expect("Failed to build ultralight config");
        platform::enable_platform_fontloader();
        platform::enable_platform_filesystem(platform_filesystem())
            .expect("Failed to get platform filesystem");
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
            views: Vec::new(),
        }
    }
}

impl Ultralight {
    /// Creates a new Ultralight adapter
    /// to enable gpu acceleration use feature ultralight-acceleration
    pub fn new(font: &str, scale: f64) -> Self {
        #[cfg(feature = "ultralight-acceleration")]
        let acceration = true;
        #[cfg(not(feature = "ultralight-acceleration"))]
        let acceration = false;

        if acceration {
            let (sender, mut receiver) = ul_next::gpu_driver::glium::create_gpu_driver()
                .expect("Failed to create ultralight glium gpu driver");
            platform::set_gpu_driver(sender);
        }

        Self {
            view_config: view::ViewConfig::start()
                .initial_device_scale(scale)
                .font_family_standard(font)
                .is_accelerated(acceration)
                .build()
                .unwrap(),
            ..Default::default()
        }
    }

    fn get_view(&self, id: ViewId) -> Option<&View> {
        self.views.iter().find(|&view| view.id == id)
    }

    fn get_view_mut(&mut self, id: ViewId) -> Option<&mut View> {
        self.views.iter_mut().find(|view| view.id == id)
    }
}

impl Engine for Ultralight {
    fn update(&mut self) {
        self.renderer.update();
    }

    fn render(&mut self, size: Size<u32>) {
        self.renderer.render();

        // for each view save new view
        // TODO: could probably be optimized for performace
        for view in self.views.iter_mut() {
            if let Some(pixels) = view.surface.lock_pixels() {
                view.last_frame =
                    ImageInfo::new(pixels.to_vec(), PixelFormat::Bgra, size.width, size.height);
            }
        }
    }

    fn request_render(&mut self, id: ViewId, size: Size<u32>) -> Option<()> {
        self.get_view_mut(id)?.view.set_needs_paint(true);

        for view in self.views.iter_mut().filter(|view| view.id == id) {
            if let Some(pixels) = view.surface.lock_pixels() {
                view.last_frame =
                    ImageInfo::new(pixels.to_vec(), PixelFormat::Bgra, size.width, size.height);
                return Some(());
            }
        }
        None
    }

    fn new_view(&mut self, size: Size<u32>) -> Option<ViewId> {
        let id = rand::thread_rng().gen();

        let view = self
            .renderer
            // TODO: debug why new views are slanted unless do + 10/ - 10
            // maybe causes the fuzzyness
            .create_view(size.width + 10, size.height - 10, &self.view_config, None)?;

        let surface = view.surface()?;

        // RGBA - ensure it has the right diamentions
        debug_assert!(surface.row_bytes() / size.width == 4);

        let cursor = Arc::new(RwLock::new(mouse::Interaction::Idle));
        let cb_cursor = cursor.clone();
        view.set_change_cursor_callback(move |_view, cursor_update| {
            *cb_cursor.write().expect("Failed to write cursor status") = match cursor_update {
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

        let view = View {
            id,
            surface,
            view,
            cursor,
            last_frame: ImageInfo::default(),
        };
        self.views.push(view);
        Some(id)
    }

    fn remove_view(&mut self, id: ViewId) -> Option<()> {
        let views = self.views.len();
        self.views.retain(|view| view.id != id);
        if self.views.len() == views {
            None
        } else {
            Some(())
        }
    }

    fn goto(&mut self, id: ViewId, page_type: PageType) -> Option<()> {
        match page_type {
            PageType::Url(url) => self.get_view_mut(id)?.view.load_url(&url).ok()?,
            PageType::Html(html) => self.get_view_mut(id)?.view.load_html(&html).ok()?,
        }
        Some(())
    }

    fn focus(&mut self) {
        self.views.iter().for_each(|view| view.view.focus());
    }

    fn unfocus(&self) {
        self.views.iter().for_each(|view| view.view.unfocus());
    }

    fn resize(&mut self, size: Size<u32>) {
        self.views.iter().for_each(|view| {
            view.view.resize(size.width, size.height);
            view.surface.resize(size.width, size.height);
        })
    }

    fn handle_keyboard_event(&mut self, id: ViewId, event: keyboard::Event) -> Option<()> {
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
            self.get_view_mut(id)?.view.fire_key_event(key_event);
        }
        Some(())
    }

    fn handle_mouse_event(&mut self, id: ViewId, point: Point, event: mouse::Event) -> Option<()> {
        match event {
            mouse::Event::ButtonReleased(mouse::Button::Forward) => self.go_forward(id),
            mouse::Event::ButtonReleased(mouse::Button::Back) => self.go_back(id),
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                self.get_view_mut(id)?.view.fire_mouse_event(
                    MouseEvent::new(
                        ul_next::event::MouseEventType::MouseDown,
                        point.x as i32,
                        point.y as i32,
                        ul_next::event::MouseButton::Left,
                    )
                    .expect("Ultralight failed to fire mouse input"),
                );
                Some(())
            }
            mouse::Event::ButtonReleased(mouse::Button::Left) => {
                self.get_view_mut(id)?.view.fire_mouse_event(
                    MouseEvent::new(
                        ul_next::event::MouseEventType::MouseUp,
                        point.x as i32,
                        point.y as i32,
                        ul_next::event::MouseButton::Left,
                    )
                    .expect("Ultralight failed to fire mouse input"),
                );
                Some(())
            }
            mouse::Event::ButtonPressed(mouse::Button::Right) => {
                self.get_view_mut(id)?.view.fire_mouse_event(
                    MouseEvent::new(
                        ul_next::event::MouseEventType::MouseDown,
                        point.x as i32,
                        point.y as i32,
                        ul_next::event::MouseButton::Right,
                    )
                    .expect("Ultralight failed to fire mouse input"),
                );
                Some(())
            }
            mouse::Event::ButtonReleased(mouse::Button::Right) => {
                self.get_view_mut(id)?.view.fire_mouse_event(
                    MouseEvent::new(
                        ul_next::event::MouseEventType::MouseUp,
                        point.x as i32,
                        point.y as i32,
                        ul_next::event::MouseButton::Right,
                    )
                    .expect("Ultralight failed to fire mouse input"),
                );
                Some(())
            }
            mouse::Event::CursorMoved { position: _ } => {
                self.get_view_mut(id)?.view.fire_mouse_event(
                    MouseEvent::new(
                        ul_next::event::MouseEventType::MouseMoved,
                        point.x as i32,
                        point.y as i32,
                        ul_next::event::MouseButton::None,
                    )
                    .expect("Ultralight failed to fire mouse input"),
                );
                Some(())
            }
            mouse::Event::WheelScrolled { delta } => self.scroll(id, delta),
            mouse::Event::CursorLeft => {
                self.unfocus();
                Some(())
            }
            mouse::Event::CursorEntered => {
                self.focus();
                Some(())
            }
            _ => Some(()),
        }
    }

    fn refresh(&mut self, id: ViewId) -> Option<()> {
        self.get_view_mut(id)?.view.reload();
        Some(())
    }

    fn go_forward(&mut self, id: ViewId) -> Option<()> {
        self.get_view_mut(id)?.view.go_forward();
        Some(())
    }

    fn go_back(&mut self, id: ViewId) -> Option<()> {
        self.get_view_mut(id)?.view.go_back();
        Some(())
    }

    fn scroll(&mut self, id: ViewId, delta: mouse::ScrollDelta) -> Option<()> {
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
        self.get_view_mut(id)?.view.fire_scroll_event(scroll_event);
        Some(())
    }

    fn get_url(&self, id: ViewId) -> EngineResult<String> {
        if let Some(view) = self.get_view(id) {
            match view.view.url() {
                Ok(url) => EngineResult::Success(url),
                Err(_) => EngineResult::NotLoaded,
            }
        } else {
            EngineResult::IdDoesNotExist
        }
    }

    fn get_title(&self, id: ViewId) -> EngineResult<String> {
        if let Some(view) = self.get_view(id) {
            match view.view.title() {
                Ok(title) => EngineResult::Success(title),
                Err(_) => EngineResult::NotLoaded,
            }
        } else {
            EngineResult::IdDoesNotExist
        }
    }

    fn get_cursor(&self, id: ViewId) -> EngineResult<mouse::Interaction> {
        if let Some(view) = self.get_view(id) {
            match view.cursor.read() {
                Ok(cursor) => EngineResult::Success(*cursor),
                Err(_) => EngineResult::NotLoaded,
            }
        } else {
            EngineResult::IdDoesNotExist
        }
    }

    fn get_view(&self, id: ViewId) -> EngineResult<&ImageInfo> {
        if let Some(view) = self.get_view(id) {
            if self.get_view(id).unwrap().view.is_loading() {
                EngineResult::NotLoaded
            } else {
                EngineResult::Success(&view.last_frame)
            }
        } else {
            EngineResult::IdDoesNotExist
        }
    }
}

fn platform_filesystem() -> PathBuf {
    let env = var("ULTRALIGHT_RESOURCES_DIR");
    let resources_path: PathBuf = match env {
        Ok(env) => PathBuf::from_str(&env)
            .expect("Failed to get path from ultralight resources enviroment varible"),
        Err(_) => {
            // env not set - check if its been symlinked by build.rs
            match Path::new("./resources").exists() {
                    true => Path::new("./resources").to_owned(),
                    false => panic!("ULTRALIGHT_RESOURCES_DIR was not set and ultralight-resources feature was not enabled"),
                }
        }
    };
    assert!(Path::new(&resources_path).join("cacert.pem").exists());
    assert!(Path::new(&resources_path).join("icudt67l.dat").exists());
    resources_path
        .parent() // leaves resources directory
        .expect("resources path needs to point to the resources directory")
        .into()
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
