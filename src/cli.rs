use serde::{Serialize, Deserialize};
use serde_json;

use brdgme_game::{Gamer, Log, Renderer, GameError};
use brdgme_markup;

use std::fmt::Debug;
use std::io::{Read, Write};

#[derive(Deserialize, Debug)]
enum Request<T: Gamer + Debug + Clone + Serialize + Deserialize> {
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
struct CliLog {
    content: String,
    at: String,
    public: bool,
    to: Vec<usize>,
}

impl CliLog {
    fn from_log(log: &Log) -> CliLog {
        CliLog {
            content: brdgme_markup::to_string(&log.content),
            at: format!("{}", log.at.format("%+")),
            public: log.public,
            to: log.to.clone(),
        }
    }

    fn from_logs(logs: &[Log]) -> Vec<CliLog> {
        logs.iter().map(CliLog::from_log).collect()
    }
}

#[derive(Serialize)]
struct GameResponse<T: Gamer + Debug + Clone + Serialize + Deserialize> {
    game: T,
    is_finished: bool,
    whose_turn: Vec<usize>,
    winners: Vec<usize>,
    eliminated: Vec<usize>,
}

#[derive(Serialize)]
enum Response<T: Gamer + Debug + Clone + Serialize + Deserialize> {
    New {
        game: GameResponse<T>,
        logs: Vec<CliLog>,
    },
    Play {
        game: GameResponse<T>,
        logs: Vec<CliLog>,
        remaining_command: String,
    },
    Render { render: String },
    UserError { message: String },
    SystemError { message: String },
}

impl<T: Gamer + Debug + Clone + Serialize + Deserialize> GameResponse<T> {
    fn from_gamer(gamer: &T) -> GameResponse<T> {
        GameResponse {
            game: gamer.clone(),
            is_finished: gamer.is_finished(),
            whose_turn: gamer.whose_turn(),
            winners: gamer.winners(),
            eliminated: gamer.eliminated(),
        }
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
                                            Response::SystemError::<T> {
                                                message: message.to_string(),
                                            }
                                        }
                                        Ok(Request::New { players }) => handle_new(players),
                                        Ok(Request::Play { player, command, names, game }) => {
                                            handle_play(player, &command, &names, &game)
                                        }
                                        Ok(Request::Render { player, game }) => {
                                            handle_render(player, &game)
                                        }
                                    })
                     .unwrap())
            .unwrap();
}

fn handle_new<T>(players: usize) -> Response<T>
    where T: Gamer + Debug + Clone + Serialize + Deserialize
{
    match T::new(players) {
        Ok((game, logs)) => {
            Response::New {
                game: GameResponse::from_gamer(&game),
                logs: CliLog::from_logs(&logs),
            }
        }
        Err(GameError::Internal(e)) => Response::SystemError { message: e.to_string() },
        Err(e) => Response::UserError { message: e.to_string() },
    }
}

fn handle_play<T>(player: usize, command: &str, names: &[String], game: &T) -> Response<T>
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
                    return Response::Play {
                               game: GameResponse::from_gamer(&game),
                               logs: CliLog::from_logs(&all_logs),
                               remaining_command: remaining_trimmed.to_string(),
                           };
                }
                remaining_command = remaining_trimmed.to_string();
            }
            Err(GameError::Internal(e)) => return Response::SystemError { message: e.to_string() },
            Err(e) => return Response::UserError { message: e.to_string() },
        };
    }
}

fn handle_render<T>(player: Option<usize>, game: &T) -> Response<T>
    where T: Gamer + Debug + Clone + Serialize + Deserialize
{
    Response::Render { render: brdgme_markup::to_string(&game.pub_state(player).render()) }
}
