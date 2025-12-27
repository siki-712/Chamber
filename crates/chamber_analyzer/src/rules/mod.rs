//! Analysis rules for ABC notation.
//!
//! Each rule checks for a specific semantic issue.

pub mod suspicious_duration;
pub mod unknown_decoration;
pub mod unusual_octave;

pub use suspicious_duration::SuspiciousDuration;
pub use unknown_decoration::UnknownDecoration;
pub use unusual_octave::UnusualOctave;
