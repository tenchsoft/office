//! # tench-ui
//!
//! Self-built retained-mode UI framework for Tench products.
//!
//! Inspired by Masonry (widget tree), egui (painter API), and Vello (GPU rendering).
//! All widgets, layout, and event handling are implemented in-house.
//! External dependencies are limited to GPU rendering (Vello/wgpu),
//! text shaping (Parley), and accessibility (AccessKit).

pub mod anim;
pub mod core;
pub mod layout;
pub mod platform;
pub mod render;
pub mod text;
pub mod theme;
pub mod widgets;

// Re-export core types for convenience
pub use core::events::{PointerEvent, TextEvent, WindowEvent};
pub use core::types::{Color, WidgetId};
pub use core::widget::{
    AccessCtx, AccessRole, AccessibilityNode, EventCtx, GlobalState, LayoutCtx, MeasureCtx,
    PaintCtx, Widget, WidgetPod,
};
pub use tench_ui_automation_core::{
    describe_selector, find_node, find_nodes, format_capture_report, format_tree_report,
    interactive_node_inventory, node_inventory, nodes_at_point, walk_nodes, UiAutomationAction,
    UiAutomationCapture, UiAutomationCaptureRequest, UiAutomationError, UiAutomationKey,
    UiAutomationModifiers, UiAutomationNode, UiAutomationNodeSummary, UiAutomationPoint,
    UiAutomationRect, UiAutomationSelector,
};

// Re-export kurbo types used throughout the framework
pub use kurbo::{Point, Rect, Size};

// Re-export widgets
pub use widgets::{
    data_grid_hit_test, data_grid_visible_range, flatten_visible_tree, virtual_tree_hit_test,
    virtual_tree_search, virtual_tree_visible_range, visual_surface_frame, visual_surface_hit_test,
    visual_surface_timeline_position, visual_surface_unit_rect, AppEntry, AppGrid, Avatar, Badge,
    Breadcrumb, Button, Card, ChatBubble, Checkbox, ColorPicker, ContextMenu, ContextMenuItem,
    CrossAxisAlignment, DataGrid, DataGridCell, DataGridColumn, DataGridHit, DataGridHitTarget,
    DataGridRow, DataGridViewport, DataGridVisibleRange, Drawer, DrawerSide, Dropdown, Flex,
    FlexDirection, Grid, Label, MenuBar, Modal, PdfSurface, PdfSurfaceHit, PdfSurfaceOverlay,
    PdfSurfaceOverlayKind, PdfSurfacePage, PdfSurfaceTheme, PdfSurfaceViewport, Portal,
    ProgressBar, RadioGroup, Ruler, ScrollableList, Scrollbar, ScrollbarOrientation, SeekBar,
    Separator, SeparatorDirection, SizedBox, Spinner, Split, Table, Tabs, TextArea, TextInput,
    Toast, ToastKind, Toggle, Toolbar, ToolbarItem, TreeItem, TreeView, VirtualTree,
    VirtualTreeHit, VirtualTreeNode, VirtualTreeRow, VirtualTreeSearchMatch, VirtualTreeViewport,
    VirtualTreeVisibleRange, VisualSurface, VisualSurfaceCommand, VisualSurfaceCommandKind,
    VisualSurfaceHit, VisualSurfaceViewport, WordCounter,
};

// Re-export platform backends (feature-gated)
#[cfg(feature = "tauri")]
pub use platform::tauri::{init_tauri_ui, TauriBackend, TauriBackendState, TauriUiOptions};

#[cfg(feature = "native")]
pub use platform::native::{
    run_native, run_native_with_config, NativeApp, NativeBackend, NativeConfig,
};

// Re-export theme
pub use theme::Theme;

// Re-export external crates that product apps need
pub use kurbo;
pub use parley;
pub use peniko;
pub use vello;

/// Convenience prelude for product apps.
pub mod prelude {
    pub use crate::anim::AnimInterval;
    pub use crate::core::events::{
        LogicalKey, Modifiers, NamedKey, PointerEvent, TextEvent, WindowEvent,
    };
    pub use crate::core::types::{Color, CursorIcon, WidgetId};
    pub use crate::core::widget::{
        AccessCtx, AccessRole, AccessibilityNode, EventCtx, GlobalState, LayoutCtx, MeasureCtx,
        PaintCtx, Widget, WidgetPod, WidgetState,
    };
    pub use crate::render::{GradientDirection, ImageCache, Painter, TextCache};
    pub use crate::theme::Theme;
    pub use crate::vello::Scene;
    pub use crate::widgets::{
        data_grid_hit_test, data_grid_visible_range, flatten_visible_tree, virtual_tree_hit_test,
        virtual_tree_search, virtual_tree_visible_range, visual_surface_frame,
        visual_surface_hit_test, visual_surface_timeline_position, visual_surface_unit_rect,
        AppEntry, AppGrid, Avatar, Badge, Breadcrumb, Button, Card, ChatBubble, Checkbox,
        ColorPicker, ContextMenu, ContextMenuItem, CrossAxisAlignment, DataGrid, DataGridCell,
        DataGridColumn, DataGridHit, DataGridHitTarget, DataGridRow, DataGridViewport,
        DataGridVisibleRange, Drawer, DrawerSide, Dropdown, Flex, FlexDirection, Grid, Label,
        MenuBar, Modal, PdfSurface, PdfSurfaceHit, PdfSurfaceOverlay, PdfSurfaceOverlayKind,
        PdfSurfacePage, PdfSurfaceTheme, PdfSurfaceViewport, Portal, ProgressBar, RadioGroup,
        Ruler, ScrollableList, Scrollbar, ScrollbarOrientation, SeekBar, Separator,
        SeparatorDirection, SizedBox, Spinner, Split, Table, Tabs, TextArea, TextInput, Toast,
        ToastKind, Toggle, Toolbar, ToolbarItem, TreeItem, TreeView, VirtualTree, VirtualTreeHit,
        VirtualTreeNode, VirtualTreeRow, VirtualTreeSearchMatch, VirtualTreeViewport,
        VirtualTreeVisibleRange, VisualSurface, VisualSurfaceCommand, VisualSurfaceCommandKind,
        VisualSurfaceHit, VisualSurfaceViewport, WordCounter,
    };
    pub use crate::{
        describe_selector, find_node, find_nodes, format_capture_report, format_tree_report,
        interactive_node_inventory, node_inventory, nodes_at_point, walk_nodes, UiAutomationAction,
        UiAutomationCapture, UiAutomationCaptureRequest, UiAutomationError, UiAutomationKey,
        UiAutomationModifiers, UiAutomationNode, UiAutomationNodeSummary, UiAutomationPoint,
        UiAutomationRect, UiAutomationSelector,
    };
    pub use kurbo::{Axis, Point, Rect, Size};
}
