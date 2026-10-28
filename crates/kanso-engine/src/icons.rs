//! Kanso's icon themes without a window: the file format a theme is written
//! in, what a theme draws for an icon or a library slot, and what falls back
//! to Kanso's own Lucide icons, icon by icon.
//!
//! The desktop app draws the result with GPUI and the mobile app with the
//! shell's SVG painter; both read themes here. [`file`] reads and checks a
//! theme file and its SVGs; [`model`] holds what is read.

pub mod file;
pub mod model;
