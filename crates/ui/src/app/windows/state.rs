use super::super::DeviceFocus;
use iced::window;

#[derive(Clone, Copy, Debug)]
pub(crate) enum NativeWindow {
    Opening(window::Id),
    Ready(window::Id),
}

impl NativeWindow {
    pub(super) fn id(self) -> window::Id {
        match self {
            Self::Opening(id) | Self::Ready(id) => id,
        }
    }
    fn ready(self) -> bool {
        matches!(self, Self::Ready(_))
    }
}

#[derive(Clone, Copy, Debug)]
enum ToolWindowPhase {
    Attached { cached: Option<NativeWindow> },
    Detached { window: NativeWindow, pinned: bool },
}

impl Default for ToolWindowPhase {
    fn default() -> Self {
        Self::Attached { cached: None }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ToolWindowState {
    pub(crate) focus: DeviceFocus,
    phase: ToolWindowPhase,
}

impl ToolWindowState {
    pub(crate) fn id(self) -> Option<window::Id> {
        self.native().map(NativeWindow::id)
    }
    pub(crate) fn ready(self) -> bool {
        self.native().is_some_and(NativeWindow::ready)
    }
    pub(crate) fn detached(self) -> bool {
        matches!(self.phase, ToolWindowPhase::Detached { .. })
    }
    pub(crate) fn always_on_top(self) -> bool {
        matches!(self.phase, ToolWindowPhase::Detached { pinned: true, .. })
    }

    pub(crate) fn native(self) -> Option<NativeWindow> {
        match self.phase {
            ToolWindowPhase::Attached { cached } => cached,
            ToolWindowPhase::Detached { window, .. } => Some(window),
        }
    }

    pub(crate) fn set_native(&mut self, window: NativeWindow) {
        match &mut self.phase {
            ToolWindowPhase::Attached { cached } => *cached = Some(window),
            ToolWindowPhase::Detached {
                window: current, ..
            } => *current = window,
        }
    }

    pub(crate) fn detach(&mut self, window: NativeWindow) {
        self.phase = ToolWindowPhase::Detached {
            window,
            pinned: false,
        };
    }

    pub(crate) fn attach(&mut self) {
        self.phase = ToolWindowPhase::Attached {
            cached: self.native(),
        };
    }

    pub(crate) fn mark_ready(&mut self) {
        if let Some(id) = self.id() {
            self.set_native(NativeWindow::Ready(id));
        }
    }

    pub(crate) fn take_id(&mut self) -> Option<window::Id> {
        let id = self.id();
        self.phase = ToolWindowPhase::default();
        id
    }

    pub(crate) fn toggle_pin(&mut self) -> Option<(window::Id, bool)> {
        if let ToolWindowPhase::Detached { window, pinned } = &mut self.phase {
            *pinned = !*pinned;
            Some((window.id(), *pinned))
        } else {
            None
        }
    }

    #[cfg(test)]
    pub(crate) fn pin(&mut self) {
        if !self.always_on_top() {
            self.toggle_pin();
        }
    }
}
