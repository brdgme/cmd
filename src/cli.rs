use serde::{Serialize, Deserialize};
use serde_json;
use chrono::NaiveDateTime;

use brdgme_game::{Gamer, Log, Renderer, GameError, Status};
use brdgme_markup;

use std::fmt::Debug;
use std::io::{Read, Write};

use errors::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request<T: Gamer + Debug + Clone + Serialize + Deserialize> {
    New { players: usize },
    Play {
        player: usize,
        command: String,
        names: Vec<String>,
        game: T,
    },
    Render { player: Option<usize>, game: T },
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct GameResponse {
    pub state: String,
    pub status: Status,
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    New {
        game: GameResponse,
        logs: Vec<CliLog>,
    },
    Play {
        game: GameResponse,
        logs: Vec<CliLog>,
        remaining_command: String,
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
             serde_json::to_string(&match serde_json::from_reader::<_, Request<T>>(input) {
                                        Err(message) => {
                                            Response::SystemError { message: message.to_string() }
                                        }
                                        Ok(Request::New::<T> { players }) => {
                                            handle_new::<T>(players)
                                        }
                                        Ok(Request::Play {
                                               player,
                                               command,
                                               names,
                                               game,
                                           }) => handle_play(player, &command, &names, &game),
                                        Ok(Request::Render { player, game }) => {
                                            handle_render(player, &game)
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
        Err(GameError::Internal(e)) => Response::SystemError { message: e.to_string() },
        Err(e) => Response::UserError { message: e.to_string() },
    }
}

fn handle_play<T>(player: usize, command: &str, names: &[String], game: &T) -> Response
    where T: Gamer + Debug + Clone + Serialize + Deserialize
{
    let mut game = game.clone();
    let mut remaining_command = command.to_string();
    let mut all_logs = vec![];
    loop {
        match game.command(player, &remaining_command, names) {
            Ok((logs, remaining)) => {
                all_logs.extend(logs);
                let remaining_trimmed = remaining.trim();
                if remaining_trimmed.is_empty() || remaining_command == remaining_trimmed {
                    return GameResponse::from_gamer(&game)
                               .map(|gr| {
                                        Response::Play {
                                            game: gr,
                                            logs: CliLog::from_logs(&all_logs),
                                            remaining_command: remaining_trimmed.to_string(),
                                        }
                                    })
                               .unwrap_or_else(|e| {
                                                   Response::SystemError { message: e.to_string() }
                                               });
                }
                remaining_command = remaining_trimmed.to_string();
            }
            Err(GameError::Internal(e)) => return Response::SystemError { message: e.to_string() },
            Err(e) => return Response::UserError { message: e.to_string() },
        };
    }
}

fn handle_render<T>(player: Option<usize>, game: &T) -> Response
    where T: Gamer + Debug + Clone + Serialize + Deserialize
{
    Response::Render { render: brdgme_markup::to_string(&game.pub_state(player).render()) }
}
