//! Based on which button is held during startup, use a different interface.
//! An interface takes ownership of all the physical hardware on the board
//! and of the inter-process FIFO. It exposes a start function which loops as necessary.

/// Interface for when low C is held
pub mod chordloops;
/// Interface for when no buttons are held, no special behaviour.
pub mod sandbox;

pub trait Interface {
    fn start(self) -> !;
}
