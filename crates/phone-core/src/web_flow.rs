//! A bounded HTML flow description. This is a host layout model, not the native engine.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Flow {
    Text {
        text: String,
        heading: bool,
        bold: bool,
    },
    Control {
        element: usize,
        width: u16,
        height: u16,
    },
    Disabled {
        width: u16,
        height: u16,
        value: String,
    },
    Break {
        gap: u16,
    },
    Indent(i16),
    BoxStart(String),
    BoxEnd,
    Bullet,
}
