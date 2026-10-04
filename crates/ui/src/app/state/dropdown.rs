#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DropdownState<T = usize> {
    #[default]
    Closed,
    Open {
        highlight: Option<T>,
    },
}

impl<T: Copy> DropdownState<T> {
    pub(crate) fn is_open(self) -> bool {
        matches!(self, Self::Open { .. })
    }
    pub(crate) fn highlight(self) -> Option<T> {
        match self {
            Self::Closed => None,
            Self::Open { highlight } => highlight,
        }
    }
    pub(crate) fn set_open(&mut self, open: bool) {
        *self = if open {
            Self::Open {
                highlight: self.highlight(),
            }
        } else {
            Self::Closed
        };
    }
    pub(crate) fn set_highlight(&mut self, value: Option<T>) {
        if let Self::Open { highlight } = self {
            *highlight = value;
        }
    }
}
