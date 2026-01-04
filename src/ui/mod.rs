// UI module - rendering and input handling

pub mod keybinds;
pub mod render;

pub use keybinds::{handle_events, Action};
pub use render::render;
