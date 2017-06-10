use chrono;

use brdgme_game::bot::{Botter, Fuzzer};
use brdgme_game::Gamer;
use brdgme_rand_bot::RandBot;

use std::io::Write;

pub fn fuzz<G, B, O>(bot: B, out: &mut O)
    where G: Gamer,
          B: Botter<G>,
          O: Write
{
    let mut last_status = chrono::UTC::now().timestamp();
    let mut f = Fuzzer::<G, _>::new(bot);
    loop {
        f.next();
        let now = chrono::UTC::now().timestamp();
        if now - last_status > 1 {
            last_status = now;
            writeln!(out, "{}", f.status()).unwrap();
        }
    }
}

pub fn fuzz_rand<G, O>(out: &mut O)
    where G: Gamer,
          O: Write
{
    fuzz::<G, _, _>(RandBot {}, out);
}
