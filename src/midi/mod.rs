pub mod escape;
mod midi_to_mtxt;
mod mtxt_to_midi;
pub mod shared;

pub use midi_to_mtxt::convert_midi_to_mtxt;
pub use mtxt_to_midi::convert_mtxt_to_midi;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
