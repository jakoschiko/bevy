use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::Component;
use bevy_window::default_event_handling::DefaultEventHandling;

#[derive(Component)]
pub struct CanvasListeners {
    default_event_handling: Arc<Mutex<DefaultEventHandling>>,
}

impl CanvasListeners {
    #[cfg(target_arch = "wasm32")]
    pub fn new(
        canvas: web_sys::HtmlCanvasElement,
        default_event_handling: DefaultEventHandling,
    ) -> Self {
        let default_event_handling = Arc::new(Mutex::new(default_event_handling));
        wasm::install(canvas, default_event_handling.clone());
        Self {
            default_event_handling,
        }
    }

    pub fn set_default_event_handling(&mut self, default_event_handling: DefaultEventHandling) {
        *self.default_event_handling.lock().unwrap() = default_event_handling
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::sync::{Arc, Mutex};

    use bevy_input::{
        keyboard::{KeyCode, NativeKeyCode},
        ButtonState,
    };
    use bevy_window::default_event_handling::{DefaultEventHandling, DefaultEventRules};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use web_sys::HtmlCanvasElement;

    pub fn install(
        canvas: HtmlCanvasElement,
        default_event_handling: Arc<Mutex<DefaultEventHandling>>,
    ) {
        let keyup = {
            let default_event_handling = default_event_handling.clone();
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                if prevent_default(&default_event_handling, |rules| {
                    is_key_exception(&event, rules, false)
                }) {
                    event.prevent_default();
                }
            })
        };
        let _ = canvas.add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref());
        keyup.forget();

        let keydown = {
            let default_event_handling = default_event_handling.clone();
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                if prevent_default(&default_event_handling, |rules| {
                    is_key_exception(&event, rules, true)
                }) {
                    event.prevent_default();
                }
            })
        };
        let _ =
            canvas.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref());
        keydown.forget();

        let pointerup = {
            let canvas = canvas.clone();
            let default_event_handling = default_event_handling.clone();
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::PointerEvent| {
                if prevent_default(&default_event_handling, |_| false) {
                    event.prevent_default();
                    let _ = canvas.focus();
                }
            })
        };
        let _ = canvas
            .add_event_listener_with_callback("pointerup", pointerup.as_ref().unchecked_ref());
        pointerup.forget();

        let pointerdown = {
            let canvas = canvas.clone();
            let default_event_handling = default_event_handling.clone();
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::PointerEvent| {
                if prevent_default(&default_event_handling, |_| false) {
                    event.prevent_default();
                    let _ = canvas.focus();
                }
            })
        };
        let _ = canvas
            .add_event_listener_with_callback("pointerdown", pointerdown.as_ref().unchecked_ref());
        pointerdown.forget();

        let pointermove = {
            let canvas = canvas.clone();
            let default_event_handling = default_event_handling.clone();
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::PointerEvent| {
                if prevent_default(&default_event_handling, |_| false) {
                    event.prevent_default();
                    let _ = canvas.focus();
                }
            })
        };
        let _ = canvas
            .add_event_listener_with_callback("pointermove", pointermove.as_ref().unchecked_ref());
        pointermove.forget();

        let touchstart = {
            let default_event_handling = default_event_handling.clone();
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
                if prevent_default(&default_event_handling, |_| false) {
                    event.prevent_default();
                }
            })
        };
        let _ = canvas
            .add_event_listener_with_callback("touchstart", touchstart.as_ref().unchecked_ref());
        touchstart.forget();

        let wheel = {
            let default_event_handling = default_event_handling.clone();
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
                if prevent_default(&default_event_handling, |_| false) {
                    event.prevent_default();
                }
            })
        };
        let _ = canvas.add_event_listener_with_callback("wheel", wheel.as_ref().unchecked_ref());
        wheel.forget();

        let contextmenu = {
            let default_event_handling = default_event_handling.clone();
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
                if prevent_default(&default_event_handling, |_| false) {
                    event.prevent_default();
                }
            })
        };
        let _ = canvas
            .add_event_listener_with_callback("contextmenu", contextmenu.as_ref().unchecked_ref());
        contextmenu.forget();
    }

    fn prevent_default(
        default_event_handling: &Mutex<DefaultEventHandling>,
        is_exception: impl FnOnce(&DefaultEventRules) -> bool,
    ) -> bool {
        let default_event_handling = default_event_handling.lock().unwrap();
        let rules = default_event_handling.rules();
        rules.prevent_default ^ is_exception(&rules)
    }

    fn is_key_exception(
        event: &web_sys::KeyboardEvent,
        rules: &DefaultEventRules,
        pressed: bool,
    ) -> bool {
        let key = parse_key_code(event);
        rules.key_exceptions.iter().any(|exception| {
            exception.key == key
                && has_button_state(exception.state, pressed)
                && has_button_state(exception.ctrl_state, event.ctrl_key())
                && has_button_state(exception.meta_state, event.meta_key())
                && has_button_state(exception.shift_state, event.shift_key())
                && has_button_state(exception.alt_state, event.alt_key())
        })
    }

    fn has_button_state(state: Option<ButtonState>, pressed: bool) -> bool {
        state.is_none_or(|s| s.is_pressed() == pressed)
    }

    // Convert a `KeyboardEvent.code` into Bevy's `KeyCode`.
    //
    // `KeyboardEvent.code` represents a physical key on the keyboard. It's documented
    // here: https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/code
    //
    // Unfortunatelly there is no `TryFrom(&str)` implementation for `KeyCode` that we can use.
    // Instead we use this big match which was originally copied from winit's private function
    // `PhysicalKey::from_key_code_attribute_value`.
    fn parse_key_code(event: &web_sys::KeyboardEvent) -> KeyCode {
        match event.code().as_str() {
            "Backquote" => KeyCode::Backquote,
            "Backslash" => KeyCode::Backslash,
            "BracketLeft" => KeyCode::BracketLeft,
            "BracketRight" => KeyCode::BracketRight,
            "Comma" => KeyCode::Comma,
            "Digit0" => KeyCode::Digit0,
            "Digit1" => KeyCode::Digit1,
            "Digit2" => KeyCode::Digit2,
            "Digit3" => KeyCode::Digit3,
            "Digit4" => KeyCode::Digit4,
            "Digit5" => KeyCode::Digit5,
            "Digit6" => KeyCode::Digit6,
            "Digit7" => KeyCode::Digit7,
            "Digit8" => KeyCode::Digit8,
            "Digit9" => KeyCode::Digit9,
            "Equal" => KeyCode::Equal,
            "IntlBackslash" => KeyCode::IntlBackslash,
            "IntlRo" => KeyCode::IntlRo,
            "IntlYen" => KeyCode::IntlYen,
            "KeyA" => KeyCode::KeyA,
            "KeyB" => KeyCode::KeyB,
            "KeyC" => KeyCode::KeyC,
            "KeyD" => KeyCode::KeyD,
            "KeyE" => KeyCode::KeyE,
            "KeyF" => KeyCode::KeyF,
            "KeyG" => KeyCode::KeyG,
            "KeyH" => KeyCode::KeyH,
            "KeyI" => KeyCode::KeyI,
            "KeyJ" => KeyCode::KeyJ,
            "KeyK" => KeyCode::KeyK,
            "KeyL" => KeyCode::KeyL,
            "KeyM" => KeyCode::KeyM,
            "KeyN" => KeyCode::KeyN,
            "KeyO" => KeyCode::KeyO,
            "KeyP" => KeyCode::KeyP,
            "KeyQ" => KeyCode::KeyQ,
            "KeyR" => KeyCode::KeyR,
            "KeyS" => KeyCode::KeyS,
            "KeyT" => KeyCode::KeyT,
            "KeyU" => KeyCode::KeyU,
            "KeyV" => KeyCode::KeyV,
            "KeyW" => KeyCode::KeyW,
            "KeyX" => KeyCode::KeyX,
            "KeyY" => KeyCode::KeyY,
            "KeyZ" => KeyCode::KeyZ,
            "Minus" => KeyCode::Minus,
            "Period" => KeyCode::Period,
            "Quote" => KeyCode::Quote,
            "Semicolon" => KeyCode::Semicolon,
            "Slash" => KeyCode::Slash,
            "AltLeft" => KeyCode::AltLeft,
            "AltRight" => KeyCode::AltRight,
            "Backspace" => KeyCode::Backspace,
            "CapsLock" => KeyCode::CapsLock,
            "ContextMenu" => KeyCode::ContextMenu,
            "ControlLeft" => KeyCode::ControlLeft,
            "ControlRight" => KeyCode::ControlRight,
            "Enter" => KeyCode::Enter,
            "MetaLeft" => KeyCode::SuperLeft,
            "MetaRight" => KeyCode::SuperRight,
            "ShiftLeft" => KeyCode::ShiftLeft,
            "ShiftRight" => KeyCode::ShiftRight,
            "Space" => KeyCode::Space,
            "Tab" => KeyCode::Tab,
            "Convert" => KeyCode::Convert,
            "KanaMode" => KeyCode::KanaMode,
            "Lang1" => KeyCode::Lang1,
            "Lang2" => KeyCode::Lang2,
            "Lang3" => KeyCode::Lang3,
            "Lang4" => KeyCode::Lang4,
            "Lang5" => KeyCode::Lang5,
            "NonConvert" => KeyCode::NonConvert,
            "Delete" => KeyCode::Delete,
            "End" => KeyCode::End,
            "Help" => KeyCode::Help,
            "Home" => KeyCode::Home,
            "Insert" => KeyCode::Insert,
            "PageDown" => KeyCode::PageDown,
            "PageUp" => KeyCode::PageUp,
            "ArrowDown" => KeyCode::ArrowDown,
            "ArrowLeft" => KeyCode::ArrowLeft,
            "ArrowRight" => KeyCode::ArrowRight,
            "ArrowUp" => KeyCode::ArrowUp,
            "NumLock" => KeyCode::NumLock,
            "Numpad0" => KeyCode::Numpad0,
            "Numpad1" => KeyCode::Numpad1,
            "Numpad2" => KeyCode::Numpad2,
            "Numpad3" => KeyCode::Numpad3,
            "Numpad4" => KeyCode::Numpad4,
            "Numpad5" => KeyCode::Numpad5,
            "Numpad6" => KeyCode::Numpad6,
            "Numpad7" => KeyCode::Numpad7,
            "Numpad8" => KeyCode::Numpad8,
            "Numpad9" => KeyCode::Numpad9,
            "NumpadAdd" => KeyCode::NumpadAdd,
            "NumpadBackspace" => KeyCode::NumpadBackspace,
            "NumpadClear" => KeyCode::NumpadClear,
            "NumpadClearEntry" => KeyCode::NumpadClearEntry,
            "NumpadComma" => KeyCode::NumpadComma,
            "NumpadDecimal" => KeyCode::NumpadDecimal,
            "NumpadDivide" => KeyCode::NumpadDivide,
            "NumpadEnter" => KeyCode::NumpadEnter,
            "NumpadEqual" => KeyCode::NumpadEqual,
            "NumpadHash" => KeyCode::NumpadHash,
            "NumpadMemoryAdd" => KeyCode::NumpadMemoryAdd,
            "NumpadMemoryClear" => KeyCode::NumpadMemoryClear,
            "NumpadMemoryRecall" => KeyCode::NumpadMemoryRecall,
            "NumpadMemoryStore" => KeyCode::NumpadMemoryStore,
            "NumpadMemorySubtract" => KeyCode::NumpadMemorySubtract,
            "NumpadMultiply" => KeyCode::NumpadMultiply,
            "NumpadParenLeft" => KeyCode::NumpadParenLeft,
            "NumpadParenRight" => KeyCode::NumpadParenRight,
            "NumpadStar" => KeyCode::NumpadStar,
            "NumpadSubtract" => KeyCode::NumpadSubtract,
            "Escape" => KeyCode::Escape,
            "Fn" => KeyCode::Fn,
            "FnLock" => KeyCode::FnLock,
            "PrintScreen" => KeyCode::PrintScreen,
            "ScrollLock" => KeyCode::ScrollLock,
            "Pause" => KeyCode::Pause,
            "BrowserBack" => KeyCode::BrowserBack,
            "BrowserFavorites" => KeyCode::BrowserFavorites,
            "BrowserForward" => KeyCode::BrowserForward,
            "BrowserHome" => KeyCode::BrowserHome,
            "BrowserRefresh" => KeyCode::BrowserRefresh,
            "BrowserSearch" => KeyCode::BrowserSearch,
            "BrowserStop" => KeyCode::BrowserStop,
            "Eject" => KeyCode::Eject,
            "LaunchApp1" => KeyCode::LaunchApp1,
            "LaunchApp2" => KeyCode::LaunchApp2,
            "LaunchMail" => KeyCode::LaunchMail,
            "MediaPlayPause" => KeyCode::MediaPlayPause,
            "MediaSelect" => KeyCode::MediaSelect,
            "MediaStop" => KeyCode::MediaStop,
            "MediaTrackNext" => KeyCode::MediaTrackNext,
            "MediaTrackPrevious" => KeyCode::MediaTrackPrevious,
            "Power" => KeyCode::Power,
            "Sleep" => KeyCode::Sleep,
            "AudioVolumeDown" => KeyCode::AudioVolumeDown,
            "AudioVolumeMute" => KeyCode::AudioVolumeMute,
            "AudioVolumeUp" => KeyCode::AudioVolumeUp,
            "WakeUp" => KeyCode::WakeUp,
            "Hyper" => KeyCode::Hyper,
            "Turbo" => KeyCode::Turbo,
            "Abort" => KeyCode::Abort,
            "Resume" => KeyCode::Resume,
            "Suspend" => KeyCode::Suspend,
            "Again" => KeyCode::Again,
            "Copy" => KeyCode::Copy,
            "Cut" => KeyCode::Cut,
            "Find" => KeyCode::Find,
            "Open" => KeyCode::Open,
            "Paste" => KeyCode::Paste,
            "Props" => KeyCode::Props,
            "Select" => KeyCode::Select,
            "Undo" => KeyCode::Undo,
            "Hiragana" => KeyCode::Hiragana,
            "Katakana" => KeyCode::Katakana,
            "F1" => KeyCode::F1,
            "F2" => KeyCode::F2,
            "F3" => KeyCode::F3,
            "F4" => KeyCode::F4,
            "F5" => KeyCode::F5,
            "F6" => KeyCode::F6,
            "F7" => KeyCode::F7,
            "F8" => KeyCode::F8,
            "F9" => KeyCode::F9,
            "F10" => KeyCode::F10,
            "F11" => KeyCode::F11,
            "F12" => KeyCode::F12,
            "F13" => KeyCode::F13,
            "F14" => KeyCode::F14,
            "F15" => KeyCode::F15,
            "F16" => KeyCode::F16,
            "F17" => KeyCode::F17,
            "F18" => KeyCode::F18,
            "F19" => KeyCode::F19,
            "F20" => KeyCode::F20,
            "F21" => KeyCode::F21,
            "F22" => KeyCode::F22,
            "F23" => KeyCode::F23,
            "F24" => KeyCode::F24,
            "F25" => KeyCode::F25,
            "F26" => KeyCode::F26,
            "F27" => KeyCode::F27,
            "F28" => KeyCode::F28,
            "F29" => KeyCode::F29,
            "F30" => KeyCode::F30,
            "F31" => KeyCode::F31,
            "F32" => KeyCode::F32,
            "F33" => KeyCode::F33,
            "F34" => KeyCode::F34,
            "F35" => KeyCode::F35,
            _ => return KeyCode::Unidentified(NativeKeyCode::Unidentified),
        }
    }
}
