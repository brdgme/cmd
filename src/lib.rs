extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate term_size;
extern crate chrono;

extern crate brdgme_game;
extern crate brdgme_markup;
extern crate brdgme_color;

mod repl;
pub use repl::repl;

mod cli;
pub use cli::cli;
