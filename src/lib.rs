extern crate brdgme_game;
extern crate brdgme_markup;
extern crate brdgme_color;

use std::io::{self, Write};
use std::fmt::Debug;

use brdgme_game::{Gamer, Renderer, Commander, Log, GameError};
use brdgme_markup::ansi;
use brdgme_markup::ast::Node as N;
use brdgme_color::Style;

pub fn repl<T>(game: &mut T)
    where T: Gamer + Renderer + Commander + Debug
{
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
        if turn.len() == 0 {
            output("no player's turn, exiting");
            return;
        }
        let current_player = turn[0];
        output(&format!("\n{}\n",
                        ansi(&game.render(Some(current_player)).unwrap(), &players).unwrap()));
        let input = prompt(&format!("Enter command for {}",
                                    ansi(&vec![N::Player(current_player)], &players).unwrap()));
        match input.as_ref() {
            ":dump" | ":d" => output(&format!("{:#?}", game)),
            ":quit" | ":q" => return,
            _ => {
                match game.command(current_player, &input, &players) {
                    Ok(l) => output_logs(l, &players),
                    Err(GameError::InvalidInput(desc)) => {
                        output(&ansi(&vec![N::Bold(vec![N::Fg(brdgme_color::RED,
                                                              vec![
                            N::Text(desc),
                                                 ])])],
                                     &players)
                            .unwrap())
                    }
                    Err(e) => panic!(e),
                }
            }
        }
    }
}

fn output_logs(logs: Vec<Log>, players: &Vec<String>) {
    for l in logs {
        output(&format!("{} - {}",
                        ansi(&vec![N::Bold(vec![N::Text(format!("{}", l.at.asctime()))])],
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
