#![recursion_limit = "1024"]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate term_size;
extern crate chrono;
#[macro_use]
extern crate error_chain;

extern crate brdgme_game;
extern crate brdgme_markup;
extern crate brdgme_color;

mod repl;
pub use repl::repl;

mod cli;
pub use cli::{cli, CliLog};

mod errors {
    error_chain!{}
}
