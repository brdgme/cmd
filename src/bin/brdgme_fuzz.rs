#![allow(dead_code)]
#[macro_use]
extern crate failure;
extern crate rand;

extern crate brdgme_cmd;
extern crate brdgme_game;
extern crate brdgme_rand_bot;

use failure::Error;
use rand::{Rng, ThreadRng};

use brdgme_cmd::api;
use brdgme_cmd::requester;
use brdgme_game::command;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let client = requester::parse_args(&args).unwrap();
    let mut fuzzer = Fuzzer::new(Box::new(client)).expect("expected to create fuzzer");
    for i in 0..1000 {
        println!("{} {:?}", i, fuzzer.next());
    }
}

struct Fuzzer {
    client: Box<requester::Requester>,
    player_counts: Vec<usize>,
    names: Vec<String>,
    game: Option<FuzzGame>,
    rng: ThreadRng,
}

impl Fuzzer {
    fn new(mut client: Box<requester::Requester>) -> Result<Self, Error> {
        let player_counts = match client.request(&api::Request::PlayerCounts)? {
            api::Response::PlayerCounts { player_counts } => player_counts,
            v => bail!("invalid response to player counts request: {:?}", v),
        };
        Ok(Fuzzer {
            client,
            player_counts,
            names: vec![],
            game: None,
            rng: rand::thread_rng(),
        })
    }

    fn new_game(&mut self) -> Result<(), Error> {
        let players = *self.rng.choose(&self.player_counts).ok_or(format_err!(
            "could not get player counts from {:?}",
            self.player_counts
        ))?;
        self.names = names(players);
        match self.client.request(&api::Request::New { players })? {
            api::Response::New {
                game,
                player_renders,
                ..
            } => {
                self.game = Some(FuzzGame {
                    game,
                    player_renders,
                });
                Ok(())
            }
            v => bail!("invalid response for new game: {:?}", v),
        }
    }

    fn command(&mut self) -> Result<CommandResponse, Error> {
        let (player, command_spec, state) = match self.game {
            Some(FuzzGame {
                game:
                    api::GameResponse {
                        ref state,
                        status: brdgme_game::Status::Active { ref whose_turn, .. },
                        ..
                    },
                ref player_renders,
            }) => {
                let player = *self.rng.choose(&whose_turn).ok_or(format_err!(
                    "unable to pick active turn player from: {:?}",
                    whose_turn
                ))?;
                if player_renders.len() <= player {
                    bail!(
                        "there is no player_render for player {} in {:?}",
                        player,
                        player_renders
                    );
                }
                let player_render = &player_renders[player];
                if player_render.command_spec.is_none() {
                    bail!("player {}'s command_spec is None", player);
                }
                (player, player_render.clone().command_spec.unwrap(), state)
            }
            Some(FuzzGame {
                game:
                    api::GameResponse {
                        status: brdgme_game::Status::Finished { .. },
                        ..
                    },
                ..
            }) => bail!("the game is already finished"),
            None => bail!("there isn't a game"),
        };
        exec_rand_command(
            &mut (*self.client),
            state.to_string(),
            player,
            self.names.clone(),
            &command_spec,
            &mut self.rng,
        )
    }
}

#[derive(Debug)]
enum FuzzStep {
    Created,
    CommandOk,
    UserError,
    Finished,
    Error {
        game: Option<FuzzGame>,
        command: Option<String>,
        error: String,
    },
}

impl Iterator for Fuzzer {
    type Item = FuzzStep;

    fn next(&mut self) -> Option<Self::Item> {
        match self.game {
            Some(_) => match self.command() {
                Ok(CommandResponse::Ok(FuzzGame {
                    game:
                        api::GameResponse {
                            status: brdgme_game::Status::Finished { .. },
                            ..
                        },
                    ..
                })) => {
                    self.game = None;
                    Some(FuzzStep::Finished)
                }
                Ok(CommandResponse::Ok(game)) => {
                    self.game = Some(game);
                    Some(FuzzStep::CommandOk)
                }
                Ok(CommandResponse::UserError { .. }) => Some(FuzzStep::UserError),
                Err(e) => Some(FuzzStep::Error {
                    game: self.game.clone(),
                    command: None,
                    error: e.to_string(),
                }),
            },
            None => match self.new_game() {
                Ok(()) => Some(FuzzStep::Created),
                Err(e) => Some(FuzzStep::Error {
                    game: None,
                    command: None,
                    error: e.to_string(),
                }),
            },
        }
    }
}

fn names(players: usize) -> Vec<String> {
    (0..players).map(|p| format!("player{}", p)).collect()
}

#[derive(Clone, Debug)]
struct FuzzGame {
    game: api::GameResponse,
    player_renders: Vec<api::PlayerRender>,
}

enum CommandResponse {
    Ok(FuzzGame),
    UserError { message: String },
}

fn exec_rand_command(
    client: &mut (impl requester::Requester + ?Sized),
    game: String,
    player: usize,
    names: Vec<String>,
    command_spec: &command::Spec,
    rng: &mut ThreadRng,
) -> Result<CommandResponse, Error> {
    exec_command(
        client,
        rand_command(command_spec, &names, rng),
        game,
        player,
        names,
    )
}

fn exec_command(
    client: &mut (impl requester::Requester + ?Sized),
    command: String,
    game: String,
    player: usize,
    names: Vec<String>,
) -> Result<CommandResponse, Error> {
    match client.request(&api::Request::Play {
        command,
        game,
        names,
        player,
    })? {
        api::Response::Play {
            ref remaining_input,
            ..
        } if !remaining_input.trim().is_empty() =>
        {
            Ok(CommandResponse::UserError {
                message: "did not parse all input".to_string(),
            })
        }
        api::Response::Play {
            game,
            player_renders,
            ..
        } => Ok(CommandResponse::Ok(FuzzGame {
            game,
            player_renders,
        })),
        api::Response::UserError { message } => Ok(CommandResponse::UserError { message }),
        v @ _ => bail!(format!("{:?}", v)),
    }
}

fn rand_command(command_spec: &command::Spec, players: &[String], rng: &mut ThreadRng) -> String {
    brdgme_rand_bot::spec_to_command(command_spec, players, rng).join("")
}
