//! MTXT - Musical Text Format Library
//!
//! This library provides functionality for working with MTXT (Musical Text) format,
//! a human-readable text format for representing musical data.

pub mod file;
pub mod parser;
pub mod process;
pub mod record_parser;
pub mod transforms;
pub mod transitions;
pub mod types;
pub mod util;

#[cfg(feature = "midi")]
pub mod midi;

// Re-export commonly used types
pub use file::MtxtFile;
pub use parser::parse_mtxt;
pub use types::beat_time::BeatTime;
pub use types::note::Note;
pub use types::note::NoteTarget;
pub use types::output_record::MtxtOutputRecord;
pub use types::pitch::PitchClass;
pub use types::record::MtxtRecord;
pub use types::time_signature::TimeSignature;
pub use types::version::Version;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
