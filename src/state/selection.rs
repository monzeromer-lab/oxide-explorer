use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOp {
    Copy,
    Cut,
}

pub struct ClipboardState {
    pub operation: ClipboardOp,
    pub paths: Vec<PathBuf>,
}
