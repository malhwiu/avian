//! Utilities for implementing character controllers.

pub mod move_and_slide;
mod velocity_project;
pub mod virtual_character;

/// Re-exports common types related to character controller functionality.
pub mod prelude {
    pub use super::move_and_slide::{
        MoveAndSlide, MoveAndSlideConfig, MoveAndSlideHitData, MoveAndSlideHitResponse,
        MoveAndSlideOutput,
    };
    pub use super::virtual_character::{AutoMoveAndSlide, MoveAndSlidePlugin, VirtualCharacter};
}
