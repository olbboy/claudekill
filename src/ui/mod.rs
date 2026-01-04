// UI module - rendering and input handling

pub mod keybinds;
pub mod render;

pub use keybinds::{Action, handle_events};
pub use render::render;
