use serde::Serialize;
use serde_json;

use brdgme_color;
use term_size;

use std::io::{self, Write};
use std::fmt::Debug;
use std::borrow::Cow;
use std::iter::repeat;

use brdgme_game::{Gamer, Renderer, Log, GameError};
use brdgme_markup::{ansi, transform, Node, TNode, to_lines, from_lines};
use brdgme_color::Style;

pub fn repl<T>(original_game: &T)
    where T: Gamer + Debug + Clone + Serialize
{
    let mut game = original_game.clone();
    let mut undo_stack: Vec<T> = vec![game.clone()];
    print!("{}", Style::default().ansi());
    let mut players: Vec<String> = vec![];
    loop {
        let player = prompt(format!("Enter player {} (or blank to finish)", players.len() + 1));
        if player == "" {
            break;
        }
        players.push(player);
    }
    output_logs(game.start(players.len()).unwrap(), &players);
    while !game.is_finished() {
        let turn = game.whose_turn();
        if turn.is_empty() {
            output(&[Node::text("no player's turn, exiting")], &players);
            return;
        }
        let current_player = turn[0];
        output(&game.pub_state(Some(current_player)).render(), &players);
        let input = prompt(format!("Enter command for {}",
                                   ansi(&transform(&[Node::Player(current_player)], &players))));
        let previous = game.clone();
        match input.as_ref() {
            ":dump" | ":d" => println!("{:#?}", game),
            ":json" => println!("{}", serde_json::ser::to_string_pretty(&game).unwrap()),
            ":undo" | ":u" => {
                if let Some(u) = undo_stack.pop() {
                    game = u;
                } else {
                    output(&[Node::Bold(vec![Node::Fg(brdgme_color::RED,
                                                      vec![Node::text("No undos available")])])],
                           &players);
                }
            }
            ":quit" | ":q" => return,
            _ => {
                match game.command(current_player, &input, &players) {
                    Ok((l, _)) => {
                        undo_stack.push(previous);
                        output_logs(l, &players);
                    }
                    Err(GameError::InvalidInput(desc)) => {
                        game = previous;
                        output(&[Node::Bold(vec![Node::Fg(brdgme_color::RED,
                                                          vec![Node::text(desc)])])],
                               &players);
                    }
                    Err(e) => panic!(e),
                }
            }
        }
    }
    match game.winners().as_slice() {
        w if w.is_empty() => println!("The game is over, there are no winners"),
        w => {
            println!("The game is over, won by {}",
                     w.iter()
                         .filter(|w| **w < players.len())
                         .map(|w| players[*w].to_owned())
                         .collect::<Vec<String>>()
                         .join(", "))
        }

    }
    output(&game.pub_state(None).render(), &players);
}

fn output_logs(logs: Vec<Log>, players: &[String]) {
    for l in logs {
        let mut l_line = vec![Node::Bold(vec![Node::text(format!("{}", l.at))])];
        l_line.push(Node::text(" - "));
        l_line.extend(l.content);
        output(&l_line, players);
    }
}

fn output(nodes: &[Node], players: &[String]) {
    let (term_w, _) = term_size::dimensions().unwrap_or_default();
    print!("{}",
           ansi(&from_lines(&to_lines(&transform(nodes, players))
                                 .iter()
                                 .map(|l| {
        let l_len = TNode::len(l);
        let mut l = l.to_owned();
        if l_len < term_w {
            l.push(TNode::Bg(*Style::default().bg,
                             vec![TNode::Text(repeat(" ").take(term_w - l_len).collect())]));
        }
        l
    })
                                 .collect::<Vec<Vec<TNode>>>())));
}

fn prompt<'a, T>(s: T) -> String
    where T: Into<Cow<'a, str>>
{
    print!("{}: \x1b[K", s.into());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_owned()
}