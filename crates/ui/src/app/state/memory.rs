#[derive(Clone, Copy, Debug)]
pub(crate) struct MemoryViewport {
    pub(crate) address: Option<u16>,
    pub(crate) scroll_offset: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum MemoryView {
    #[default]
    Ram,
    Stack {
        saved: MemoryViewport,
    },
}

impl MemoryView {
    pub(crate) fn is_stack(self) -> bool {
        matches!(self, Self::Stack { .. })
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OperandReturn {
    pub(crate) address: u16,
    pub(crate) scroll_offset: f32,
}
