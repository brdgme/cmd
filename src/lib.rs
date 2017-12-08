#![recursion_limit = "1024"]

extern crate chrono;
#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate term_size;

extern crate brdgme_color;
extern crate brdgme_game;
extern crate brdgme_markup;

mod repl;
pub use repl::repl;

pub mod cli;
pub mod bot_cli;
