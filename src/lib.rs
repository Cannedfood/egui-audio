#![allow(dead_code)]
#![feature(int_roundings)]

mod fader;
mod knob;
mod util;

#[cfg(feature = "atomic-traits")]
mod atomic_wrapper;

mod db_wrapper;
mod envelope;
mod time_cursor;
mod waveform;

#[cfg(feature = "atomic-traits")]
pub use atomic_wrapper::*;
pub use db_wrapper::*;
pub use envelope::*;
pub use fader::*;
pub use knob::*;
pub use time_cursor::*;
pub use util::*;
pub use waveform::*;
