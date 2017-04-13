use serde::{Serialize, Deserialize};
use serde_json;
use chrono::NaiveDateTime;

use brdgme_game::{Gamer, Log, Renderer, Status, CommandResponse};
use brdgme_game::errors::{Error as GameError, ErrorKind as GameErrorKind};
use brdgme_markup;

use std::fmt::Debug;
use std::io::{Read, Write};

use errors::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    New { players: usize },
    Play {
        player: usize,
        command: String,
        names: Vec<String>,
        game: String,
    },
    Render { player: Option<usize>, game: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CliLog {
    pub content: String,
    pub at: NaiveDateTime,
    pub public: bool,
    pub to: Vec<usize>,
}

impl CliLog {
    fn from_log(log: &Log) -> CliLog {
        CliLog {
            content: brdgme_markup::to_string(&log.content),
            at: log.at,
            public: log.public,
            to: log.to.clone(),
        }
    }

    fn from_logs(logs: &[Log]) -> Vec<CliLog> {
        logs.iter().map(CliLog::from_log).collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameResponse {
    pub state: String,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    New {
        game: GameResponse,
        logs: Vec<CliLog>,
    },
    Play {
        game: GameResponse,
        logs: Vec<CliLog>,
        can_undo: bool,
        remaining_input: String,
    },
    Render { render: String },
    UserError { message: String },
    SystemError { message: String },
}

impl GameResponse {
    fn from_gamer<T: Gamer + Serialize>(gamer: &T) -> Result<GameResponse> {
        Ok(GameResponse {
               state: serde_json::to_string(gamer)
                   .chain_err(|| "unable to encode game state")?,
               status: gamer.status(),
           })
    }
}

pub fn cli<T, I, O>(input: I, output: &mut O)
    where T: Gamer + Debug + Clone + Serialize + Deserialize,
          I: Read,
          O: Write
{
    writeln!(output,
             "{}",
             serde_json::to_string(&match serde_json::from_reader::<_, Request>(input) {
                                        Err(message) => {
                                            Response::SystemError { message: message.to_string() }
                                        }
                                        Ok(Request::New { players }) => handle_new::<T>(players),
                                        Ok(Request::Play {
                                               player,
                                               command,
                                               names,
                                               game,
                                           }) => {
        let mut game = serde_json::from_str(&game).unwrap();
        handle_play::<T>(player, &command, &names, &mut game)
    }
                                        Ok(Request::Render { player, game }) => {
        let game = serde_json::from_str(&game).unwrap();
        handle_render::<T>(player, &game)
    }
                                    })
                     .unwrap())
            .unwrap();
}

fn handle_new<T>(players: usize) -> Response
    where T: Gamer + Debug + Clone + Serialize + Deserialize
{
    match T::new(players) {
        Ok((game, logs)) => {
            GameResponse::from_gamer(&game)
                .map(|gs| {
                         Response::New {
                             game: gs,
                             logs: CliLog::from_logs(&logs),
                         }
                     })
                .unwrap_or_else(|e| Response::SystemError { message: e.to_string() })
        }
        Err(GameError(GameErrorKind::Internal(e), _)) => {
            Response::SystemError { message: e.to_string() }
        }
        Err(e) => Response::UserError { message: e.to_string() },
    }
}

fn handle_play<T>(player: usize, command: &str, names: &[String], game: &mut T) -> Response
    where T: Gamer + Debug + Clone + Serialize + Deserialize
{
    match game.command(player, command, names) {
        Ok(CommandResponse {
               logs,
               can_undo,
               remaining_input,
           }) => {
            GameResponse::from_gamer(game)
                .map(|gr| {
                         Response::Play {
                             game: gr,
                             logs: CliLog::from_logs(&logs),
                             can_undo: can_undo,
                             remaining_input: remaining_input,
                         }
                     })
                .unwrap_or_else(|e| Response::SystemError { message: e.to_string() })
        }
        Err(GameError(GameErrorKind::Internal(e), _)) => {
            Response::SystemError { message: e.to_string() }
        }
        Err(e) => Response::UserError { message: e.to_string() },
    }
}

fn handle_render<T>(player: Option<usize>, game: &T) -> Response
    where T: Gamer + Debug + Clone + Serialize + Deserialize
{
    Response::Render { render: brdgme_markup::to_string(&game.pub_state(player).render()) }
}
