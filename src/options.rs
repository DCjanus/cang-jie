#[derive(Debug, Clone)]
pub enum TokenizerOption {
    All,
    Default { hmm: bool },
    ForSearch { hmm: bool },
    Unicode,
}
