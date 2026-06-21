#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum HexStreamFilter {
    #[default]
    All,
    Graphics,
    Text,
}

impl HexStreamFilter {
    pub(crate) fn next(self) -> Self {
        match self {
            HexStreamFilter::All => HexStreamFilter::Graphics,
            HexStreamFilter::Graphics => HexStreamFilter::Text,
            HexStreamFilter::Text => HexStreamFilter::All,
        }
    }
}
