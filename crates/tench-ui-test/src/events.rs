//! Event simulation utilities for replaying keyboard and pointer input.
//!
//! Provides builder-style helpers for constructing common event sequences
//! (clicks, key presses, typing, drag) without manually assembling each event.

use kurbo::{Point, Vec2};
use tench_ui::core::events::{
    ImeEvent, KeyboardEvent, LogicalKey, Modifiers, NamedKey, PointerButton, PointerButtonEvent,
    PointerButtons, PointerEvent, PointerMoveEvent, PointerScrollEvent, TextEvent,
};

/// Builder for simulating user input events.
///
/// `EventSimulator` provides convenience methods for creating common event
/// sequences that can be dispatched through a `TestHarness`.
pub struct EventSimulator;

impl EventSimulator {
    // --- Pointer events ---

    /// Creates a pointer down event (button press) at the given position.
    pub fn pointer_down(pos: Point, button: PointerButton) -> PointerEvent {
        let mut buttons = PointerButtons::new();
        buttons.set(button, true);
        PointerEvent::Down(PointerButtonEvent {
            button,
            pos,
            buttons,
        })
    }

    /// Creates a pointer up event (button release) at the given position.
    pub fn pointer_up(pos: Point, button: PointerButton) -> PointerEvent {
        PointerEvent::Up(PointerButtonEvent {
            button,
            pos,
            buttons: PointerButtons::new(),
        })
    }

    /// Creates a pointer move event.
    pub fn pointer_move(pos: Point, delta: Vec2) -> PointerEvent {
        PointerEvent::Move(PointerMoveEvent {
            pos,
            delta,
            buttons: PointerButtons::new(),
        })
    }

    /// Creates a scroll event at the given position.
    pub fn scroll(pos: Point, delta: Vec2) -> PointerEvent {
        PointerEvent::Scroll(PointerScrollEvent {
            pos,
            delta,
            modifiers: Modifiers::default(),
        })
    }

    /// Creates a complete click sequence: down -> up at the given position.
    /// Returns events in chronological order.
    pub fn click(pos: Point) -> Vec<PointerEvent> {
        vec![
            Self::pointer_down(pos, PointerButton::Primary),
            Self::pointer_up(pos, PointerButton::Primary),
        ]
    }

    /// Creates a double-click sequence (two clicks in succession).
    pub fn double_click(pos: Point) -> Vec<PointerEvent> {
        vec![
            Self::pointer_down(pos, PointerButton::Primary),
            Self::pointer_up(pos, PointerButton::Primary),
            Self::pointer_down(pos, PointerButton::Primary),
            Self::pointer_up(pos, PointerButton::Primary),
        ]
    }

    /// Creates a right-click sequence.
    pub fn right_click(pos: Point) -> Vec<PointerEvent> {
        vec![
            Self::pointer_down(pos, PointerButton::Secondary),
            Self::pointer_up(pos, PointerButton::Secondary),
        ]
    }

    /// Creates a drag sequence from `start` to `end` with optional steps.
    pub fn drag(start: Point, end: Point, steps: usize) -> Vec<PointerEvent> {
        let mut events = Vec::with_capacity(steps + 2);
        events.push(Self::pointer_down(start, PointerButton::Primary));

        let steps = steps.max(1);
        let delta = (end - start) / steps as f64;
        for i in 1..=steps {
            let pos = start + delta * i as f64;
            let mut buttons = PointerButtons::new();
            buttons.set(PointerButton::Primary, true);
            events.push(PointerEvent::Move(PointerMoveEvent {
                pos,
                delta,
                buttons,
            }));
        }

        events.push(Self::pointer_up(end, PointerButton::Primary));
        events
    }

    // --- Keyboard events ---

    /// Creates a keyboard event for a key press or release.
    pub fn key_event(logical_key: LogicalKey, is_pressed: bool, modifiers: Modifiers) -> TextEvent {
        TextEvent::Keyboard(KeyboardEvent {
            physical_key: 0,
            logical_key,
            is_pressed,
            is_repeat: false,
            modifiers,
        })
    }

    /// Creates a key press event.
    pub fn key_down(logical_key: LogicalKey) -> TextEvent {
        Self::key_event(logical_key, true, Modifiers::default())
    }

    /// Creates a key release event.
    pub fn key_up(logical_key: LogicalKey) -> TextEvent {
        Self::key_event(logical_key, false, Modifiers::default())
    }

    /// Creates a complete key press and release sequence.
    pub fn key_press(logical_key: LogicalKey) -> Vec<TextEvent> {
        vec![
            Self::key_down(logical_key.clone()),
            Self::key_up(logical_key),
        ]
    }

    /// Creates a key press with modifier held.
    pub fn key_press_with_mod(logical_key: LogicalKey, modifiers: Modifiers) -> Vec<TextEvent> {
        vec![
            Self::key_event(logical_key.clone(), true, modifiers),
            Self::key_event(logical_key, false, modifiers),
        ]
    }

    /// Simulates typing a string of characters, producing one key-down/key-up pair per char.
    pub fn type_text(text: &str) -> Vec<TextEvent> {
        let mut events = Vec::with_capacity(text.len() * 2);
        for ch in text.chars() {
            let key = LogicalKey::Character(ch.to_string());
            events.push(Self::key_down(key.clone()));
            events.push(Self::key_up(key));
        }
        events
    }

    /// Simulates pressing Enter.
    pub fn enter() -> Vec<TextEvent> {
        Self::key_press(LogicalKey::Named(NamedKey::Enter))
    }

    /// Simulates pressing Tab.
    pub fn tab() -> Vec<TextEvent> {
        Self::key_press(LogicalKey::Named(NamedKey::Tab))
    }

    /// Simulates pressing Escape.
    pub fn escape() -> Vec<TextEvent> {
        Self::key_press(LogicalKey::Named(NamedKey::Escape))
    }

    /// Simulates pressing Backspace.
    pub fn backspace() -> Vec<TextEvent> {
        Self::key_press(LogicalKey::Named(NamedKey::Backspace))
    }

    /// Simulates Ctrl+key combination (e.g., Ctrl+C for copy).
    pub fn ctrl_key(named: NamedKey) -> Vec<TextEvent> {
        let mods = Modifiers {
            control: true,
            ..Default::default()
        };
        Self::key_press_with_mod(LogicalKey::Named(named), mods)
    }

    /// Simulates Shift+key combination.
    pub fn shift_key(named: NamedKey) -> Vec<TextEvent> {
        let mods = Modifiers {
            shift: true,
            ..Default::default()
        };
        Self::key_press_with_mod(LogicalKey::Named(named), mods)
    }

    // --- IME events ---

    /// Creates an IME commit event (text committed from IME).
    pub fn ime_commit(text: &str) -> TextEvent {
        TextEvent::Ime(ImeEvent::Commit(text.to_string()))
    }

    // --- Window focus ---

    /// Creates a window focus change event.
    pub fn window_focus_gained() -> TextEvent {
        TextEvent::WindowFocusChange(true)
    }

    /// Creates a window focus lost event.
    pub fn window_focus_lost() -> TextEvent {
        TextEvent::WindowFocusChange(false)
    }

    // --- Clipboard ---

    /// Creates a clipboard paste event.
    pub fn paste(text: &str) -> TextEvent {
        TextEvent::ClipboardPaste(text.to_string())
    }
}
