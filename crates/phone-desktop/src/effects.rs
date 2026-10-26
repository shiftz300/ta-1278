//! Native resource choices. Only the Snake event bindings are code-verified.
#[derive(Clone, Copy, Debug)]
pub enum Cue {
    Key,
    Food,
    Over,
    Alert,
}
pub fn resource(cue: Cue) -> &'static str {
    match cue {
        Cue::Key => "key",
        Cue::Food => "snake-food",
        Cue::Over => "snake-over",
        Cue::Alert => "Message 1",
    }
}
