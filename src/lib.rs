extern crate brdgme_game;
extern crate brdgme_markup;
extern crate brdgme_color;

use std::io;
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
        output(format!("Enter player {} (or blank to finish):", players.len() + 1));
        let mut player = String::new();
        io::stdin().read_line(&mut player).unwrap();
        player = player.trim().to_owned();
        if player == "" {
            break;
        }
        players.push(player);
    }
    output_logs(game.start(players.len()).unwrap(), &players);
    while !game.is_finished() {
        let turn = game.whose_turn();
        if turn.len() == 0 {
            panic!("no player's turn");
        }
        let current_player = turn[0];
        output(format!("\n{}\n\nEnter command for {}:",
                       ansi(&game.render(Some(current_player)).unwrap(), &players).unwrap(),
                       ansi(&vec![N::Player(current_player)], &players).unwrap()));

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_owned();
        match input.as_ref() {
            ":dump" | ":d" => output(format!("{:#?}", game)),
            ":quit" | ":q" => return,
            _ => {
                match game.command(current_player, &input, &players) {
                    Ok(l) => output_logs(l, &players),
                    Err(GameError::InvalidInput(desc)) => output(desc),
                    Err(e) => panic!(e),
                }
            }
        }
    }
}

fn output_logs(logs: Vec<Log>, players: &Vec<String>) {
    for l in logs {
        output(format!("{} - {}",
                       ansi(&vec![N::Bold(vec![N::Text(format!("{}", l.at.asctime()))])],
                            &players)
                           .unwrap(),
                       ansi(&l.content, players).unwrap()));
    }
}

fn output(s: String) {
    println!("{}",
             s.split("\n")
                 .map(|l| format!("{}\x1b[K", l))
                 .collect::<Vec<String>>()
                 .join("\n"));
}
