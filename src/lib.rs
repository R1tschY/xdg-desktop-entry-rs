//! Library to parse XDG desktop files.
//!
//! More information about desktop files and the "Desktop Entry Specification" is available on:
//! - http://freedesktop.org/wiki/Specifications/desktop-entry-spec
//! - http://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html


#[macro_use(eof, error_position)] extern crate nom;
#[cfg(test)] #[macro_use] extern crate assert_matches;
#[cfg(test)] #[macro_use] extern crate indoc;
#[cfg(test)] #[macro_use] extern crate maplit;

mod parser;
mod errors;
mod desktop_entry;
mod locale;
mod discover;
mod utils;

pub use errors::*;
pub use desktop_entry::*;
pub use discover::*;