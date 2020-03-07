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