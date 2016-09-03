extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

extern crate brdgme_game;
extern crate brdgme_markup;
extern crate brdgme_color;

use serde::Serialize;

use std::io::{self, Write};
use std::fmt::Debug;

use brdgme_game::{Gamer, Renderer, Commander, Log, GameError};
use brdgme_markup::ansi;
use brdgme_markup::ast::Node as N;
use brdgme_color::Style;

pub fn repl<T>(original_game: &T)
    where T: Gamer + Renderer + Commander + Debug + Clone + Serialize
{
    let mut game = original_game.clone();
    let mut undo_stack: Vec<T> = vec![game.clone()];
    print!("{}", Style::default().ansi());
    let mut players: Vec<String> = vec![];
    loop {
        let player = prompt(&format!("Enter player {} (or blank to finish)", players.len() + 1));
        if player == "" {
            break;
        }
        players.push(player);
    }
    output_logs(game.start(players.len()).unwrap(), &players);
    while !game.is_finished() {
        let turn = game.whose_turn();
        if turn.is_empty() {
            output("no player's turn, exiting");
            return;
        }
        let current_player = turn[0];
        output(&format!("\n{}\n",
                        ansi(&game.render(Some(current_player)).unwrap(), &players).unwrap()));
        let input = prompt(&format!("Enter command for {}",
                                    ansi(&[N::Player(current_player)], &players).unwrap()));
        let previous = game.clone();
        match input.as_ref() {
            ":dump" | ":d" => output(&format!("{:#?}", game)),
            ":yaml" => output(&serde_yaml::ser::to_string(&game).unwrap()),
            ":json" => output(&serde_json::ser::to_string_pretty(&game).unwrap()),
            ":undo" | ":u" => {
                if let Some(u) = undo_stack.pop() {
                    game = u;
                } else {
                    output(&ansi(&[N::Bold(vec![N::Fg(brdgme_color::RED,
                                                          vec![
                                                              N::Text("No undos available".to_string()),
                                                          ])])],
                                 &players)
                           .unwrap());
                }
            },
            ":quit" | ":q" => return,
            _ => {
                match game.command(current_player, &input, &players) {
                    Ok((l, _)) => {
                        undo_stack.push(previous);
                        output_logs(l, &players);
                    }
                    Err(GameError::InvalidInput(desc)) => {
                        game = previous;
                        output(&ansi(&[N::Bold(vec![N::Fg(brdgme_color::RED,
                                                              vec![
                            N::Text(desc),
                                                 ])])],
                                     &players)
                            .unwrap());
                    }
                    Err(e) => panic!(e),
                }
            }
        }
    }
}

fn output_logs(logs: Vec<Log>, players: &[String]) {
    for l in logs {
        output(&format!("{} - {}",
                        ansi(&[N::Bold(vec![N::Text(format!("{}", l.at.asctime()))])],
                             &players)
                            .unwrap(),
                        ansi(&l.content, players).unwrap()));
    }
}

fn output(s: &str) {
    println!("{}",
             s.split("\n")
                 .map(|l| format!("{}\x1b[K", l))
                 .collect::<Vec<String>>()
                 .join("\n"));
}

fn prompt(s: &str) -> String {
    print!("{}: \x1b[K", s);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_owned()
}
