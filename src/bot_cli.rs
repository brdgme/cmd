use serde::{Serialize, Deserialize};
use serde_json;

use brdgme_game::Gamer;
use brdgme_game::bot::Botter;
use brdgme_game::command::Specs as CommandSpecs;

use std::fmt::Debug;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub player: usize,
    pub pub_state: String,
    pub players: Vec<String>,
    pub command_spec: CommandSpecs,
}

pub type Response = Vec<String>;

pub fn cli<G, B, I, O>(input: I, output: &mut O)
    where G: Gamer + Debug + Clone + Serialize + Deserialize,
          B: Botter<G>,
          I: Read,
          O: Write
{
    let request = serde_json::from_reader::<_, Request>(input).unwrap();
    let pub_state: G::PubState = serde_json::from_str(&request.pub_state).unwrap();
    writeln!(output,
             "{}",
             serde_json::to_string(&B::commands(request.player,
                                                &pub_state,
                                                &request.players,
                                                &request.command_spec))
                     .unwrap())
            .unwrap();
}
