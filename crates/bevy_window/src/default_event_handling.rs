//! Utilities for controlling the default event handling on web platforms.
//!
//! On a canvas, the default event handling can be disabled by calling
//! [`preventDefault()`](https://developer.mozilla.org/en-US/docs/Web/API/Event/preventDefault)
//! when the event occurs. But Bevy's input API doesn't provide any possibility to call this
//! function. This module tries to fill this gap.
//!
//! The struct [`DefaultEventHandling`] allows to enable/disable default event handling
//! for specific events. You can define a general rule with
//! [`DefaultEventHandling::prevent_default`] and then add exceptions to that.
//! The rules can be applied to the canvas by setting
//! [`Window::default_event_handling`](crate::Window::default_event_handling).
//!
//! These rules have no effect on non-web platforms.

use std::sync::Arc;

use bevy_input::{keyboard::KeyCode, ButtonState};
use bevy_platform::collections::HashSet;
use bevy_reflect::Reflect;

const CLIPBOARD_EXCEPTIONS: &[KeyEventException] = &[
    KeyEventException::new(KeyCode::KeyC).ctrl(true),
    KeyEventException::new(KeyCode::KeyC).meta(true),
    KeyEventException::new(KeyCode::KeyX).ctrl(true),
    KeyEventException::new(KeyCode::KeyX).meta(true),
    KeyEventException::new(KeyCode::KeyV).ctrl(true),
    KeyEventException::new(KeyCode::KeyV).meta(true),
];

/// Allows to enable/disable default event handling for specific events.
#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct DefaultEventHandling {
    // Use use an Arc for copy-on-write. This allows bevy_winit to cheaply copy this setting
    // when handling a Window change.
    rules: Arc<DefaultEventRules>,
}

impl Default for DefaultEventHandling {
    fn default() -> Self {
        Self::prevent_default(true)
    }
}

impl DefaultEventHandling {
    /// Enable/disable default event handling for all events.
    pub fn prevent_default(prevent_default: bool) -> Self {
        Self {
            rules: Arc::new(DefaultEventRules {
                prevent_default,
                key_exceptions: Default::default(),
            }),
        }
    }

    /// Return all defiend rules.
    pub fn rules(&self) -> &DefaultEventRules {
        &self.rules
    }

    /// Add an exception for key events to the general rule defined by [`prevent_default`].
    pub fn exception_for_key(mut self, exception: KeyEventException) -> Self {
        let rules = Arc::make_mut(&mut self.rules);
        rules.key_exceptions.insert(exception);
        self
    }

    /// Add an exception for clipboard key events to the general rule defined by
    /// [`prevent_default`].
    ///
    /// Clipboard key events include copy, cut and paste.
    pub fn exceptions_for_clipboard_keys(mut self) -> Self {
        for &exception in CLIPBOARD_EXCEPTIONS {
            self = self.exception_for_key(exception);
        }
        self
    }
}

/// Rules defined by [`DefaultEventHandling`].
#[derive(Clone, Debug, PartialEq, Reflect)]
#[non_exhaustive]
pub struct DefaultEventRules {
    /// Whether the default event handling is enable/disable for all events.
    ///
    /// Added exceptions will invert this setting for certain events.
    pub prevent_default: bool,
    /// Exceptions for key events
    pub key_exceptions: HashSet<KeyEventException>,
}

/// Defines an exception for key events.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub struct KeyEventException {
    /// The button that triggerd the event.
    pub key: KeyCode,
    /// The button state of the event. `None` means any state.
    pub state: Option<ButtonState>,
    /// The control key state of the event. `None` means any state.
    pub ctrl_state: Option<ButtonState>,
    /// The meta key state of the event. `None` means any state.
    pub meta_state: Option<ButtonState>,
    /// The shift key state of the event. `None` means any state.
    pub shift_state: Option<ButtonState>,
    /// The alt key state of the event. `None` means any state.
    pub alt_state: Option<ButtonState>,
}

impl KeyEventException {
    /// Apply the exception only for events triggered by the given key.
    pub const fn new(key: KeyCode) -> Self {
        Self {
            key,
            state: None,
            ctrl_state: None,
            meta_state: None,
            shift_state: None,
            alt_state: None,
        }
    }

    /// Apply the exception only for events with the given key state.
    pub const fn pressed(mut self, pressed: bool) -> Self {
        self.state = Some(button_state(pressed));
        self
    }

    /// Apply the exception only for events with the given control key state.
    pub const fn ctrl(mut self, pressed: bool) -> Self {
        self.ctrl_state = Some(button_state(pressed));
        self
    }

    /// Apply the exception only for events with the given meta key state.
    pub const fn meta(mut self, pressed: bool) -> Self {
        self.meta_state = Some(button_state(pressed));
        self
    }

    /// Apply the exception only for events with the given shift key state.
    pub const fn shift(mut self, pressed: bool) -> Self {
        self.shift_state = Some(button_state(pressed));
        self
    }

    /// Apply the exception only for events with the given alt key state.
    pub const fn alt(mut self, pressed: bool) -> Self {
        self.alt_state = Some(button_state(pressed));
        self
    }
}

const fn button_state(pressed: bool) -> ButtonState {
    if pressed {
        ButtonState::Pressed
    } else {
        ButtonState::Released
    }
}
