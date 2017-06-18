use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;

use brdgme_game::Gamer;
use brdgme_game::bot::Botter;
use brdgme_game::command::Spec as CommandSpec;

use std::fmt::Debug;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub player: usize,
    pub pub_state: String,
    pub players: Vec<String>,
    pub command_spec: CommandSpec,
}

pub type Response = Vec<String>;

pub fn cli<G, B, I, O>(bot: &mut B, input: I, output: &mut O)
where
    G: Gamer + Debug + Clone + Serialize + DeserializeOwned,
    B: Botter<G>,
    I: Read,
    O: Write,
{
    let request = serde_json::from_reader::<_, Request>(input).unwrap();
    let pub_state: G::PubState = serde_json::from_str(&request.pub_state).unwrap();
    writeln!(
        output,
        "{}",
        serde_json::to_string(&bot.commands(
            request.player,
            &pub_state,
            &request.players,
            &request.command_spec,
        )).unwrap()
    ).unwrap();
}
