//! Events dispatched to widgets.

use kurbo::{Point, Rect, Vec2};

/// A pointer (mouse/touch/pen) event.
#[derive(Debug, Clone)]
pub enum PointerEvent {
    /// Pointer moved.
    Move(PointerMoveEvent),
    /// Pointer button pressed.
    Down(PointerButtonEvent),
    /// Pointer button released.
    Up(PointerButtonEvent),
    /// Mouse wheel or trackpad scroll.
    Scroll(PointerScrollEvent),
    /// Pointer entered the widget's bounds.
    Enter,
    /// Pointer left the widget's bounds.
    Leave,
}

/// Data for a pointer move event.
#[derive(Debug, Clone)]
pub struct PointerMoveEvent {
    /// Position in window coordinates.
    pub pos: Point,
    /// Movement delta since last move.
    pub delta: Vec2,
    /// Which buttons are currently pressed.
    pub buttons: PointerButtons,
}

/// Data for a pointer button press/release event.
#[derive(Debug, Clone)]
pub struct PointerButtonEvent {
    /// Which button was pressed/released.
    pub button: PointerButton,
    /// Position in window coordinates.
    pub pos: Point,
    /// Which buttons are currently pressed.
    pub buttons: PointerButtons,
}

/// Data for a scroll event.
#[derive(Debug, Clone)]
pub struct PointerScrollEvent {
    /// Position in window coordinates.
    pub pos: Point,
    /// Scroll delta in pixels.
    pub delta: Vec2,
    /// Modifier flags.
    pub modifiers: Modifiers,
}

/// Mouse/pointer buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Other(u16),
}

/// Set of currently pressed buttons.
#[derive(Debug, Clone, Default)]
pub struct PointerButtons(u8);

impl PointerButtons {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn set(&mut self, button: PointerButton, pressed: bool) {
        let bit = match button {
            PointerButton::Primary => 1,
            PointerButton::Secondary => 2,
            PointerButton::Middle => 4,
            PointerButton::Other(_) => 8,
        };
        if pressed {
            self.0 |= bit;
        } else {
            self.0 &= !bit;
        }
    }

    pub fn has(&self, button: PointerButton) -> bool {
        let bit = match button {
            PointerButton::Primary => 1,
            PointerButton::Secondary => 2,
            PointerButton::Middle => 4,
            PointerButton::Other(_) => 8,
        };
        self.0 & bit != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

/// A text-related event (keyboard, IME, focus).
#[derive(Debug, Clone)]
pub enum TextEvent {
    /// A keyboard event.
    Keyboard(KeyboardEvent),
    /// An IME event.
    Ime(ImeEvent),
    /// The window gained or lost focus.
    WindowFocusChange(bool),
    /// Text pasted from clipboard.
    ClipboardPaste(String),
}

/// A keyboard event.
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    /// Physical key code.
    pub physical_key: u32,
    /// Logical key.
    pub logical_key: LogicalKey,
    /// Whether the key is pressed or released.
    pub is_pressed: bool,
    /// Whether this is a repeat event.
    pub is_repeat: bool,
    /// Modifier flags.
    pub modifiers: Modifiers,
}

/// Logical key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalKey {
    Character(String),
    Named(NamedKey),
    Unidentified,
}

/// Named keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedKey {
    Enter,
    Tab,
    Backspace,
    Delete,
    Escape,
    Space,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Shift,
    Control,
    Alt,
    Super,
    CapsLock,
    F(u8),
}

/// IME event.
#[derive(Debug, Clone)]
pub enum ImeEvent {
    Enabled,
    Disabled,
    Commit(String),
    Preedit {
        text: String,
        cursor: Option<(usize, usize)>,
    },
}

/// Keyboard modifier flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
}

/// A window-level event.
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// The window was resized.
    Resize { width: u32, height: u32 },
    /// The DPI scale factor changed.
    ScaleFactor(f64),
    /// An animation frame should run.
    AnimFrame(u64),
    /// The window gained or lost focus.
    Focused(bool),
    /// The window was destroyed.
    Destroyed,
    /// Files were dragged and dropped onto the window.
    FileDrop { paths: Vec<String> },
}

/// Changes to widget state, generated internally by the framework.
#[derive(Debug, Clone)]
pub enum Update {
    /// Widget was added to the tree.
    WidgetAdded,
    /// Widget's disabled state changed.
    DisabledChanged(bool),
    /// Widget's hovered state changed.
    HoveredChanged(bool),
    /// Widget's active (pressed) state changed.
    ActiveChanged(bool),
    /// Widget's focus state changed.
    FocusChanged(bool),
    /// A descendant's focus state changed.
    ChildFocusChanged(bool),
    /// The widget should scroll to show a region.
    RequestScrollTo(Rect),
}

/// An action emitted by a widget, to be handled by ancestors.
#[derive(Debug)]
pub struct Action {
    /// The type-erased action payload.
    pub payload: Box<dyn std::any::Any + Send>,
}

impl Action {
    /// Creates a new action from a payload.
    pub fn new(payload: impl std::any::Any + Send) -> Self {
        Self {
            payload: Box::new(payload),
        }
    }

    /// Downcasts the action payload to a specific type.
    pub fn downcast<T: 'static>(self) -> Option<T> {
        self.payload.downcast::<T>().ok().map(|b| *b)
    }

    /// Checks if the action is of a specific type.
    pub fn is<T: 'static>(&self) -> bool {
        self.payload.is::<T>()
    }
}
