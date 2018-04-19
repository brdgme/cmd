extern crate brdgme_cmd;

use std::env;

use brdgme_cmd::repl;
use brdgme_cmd::requester::local::LocalRequester;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut client: LocalRequester = match args[1].as_ref() {
        "local" => LocalRequester::new(&args[2]),
        _ => panic!("expected one of 'local'"),
    };
    repl(&mut client);
}
