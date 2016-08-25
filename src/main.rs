extern crate brdgme_markup;

use brdgme_markup::parse;
use brdgme_markup::render::ansi;

fn main() {
    println!("{}",
             ansi::render(&parse("blah {{#b}}blah{{/b}} blah {{#bg red}}{{#fg \
                                  green}}CHRISTMAS{{/fg}}{{/bg}} {{player 0}} {{player 1}}")
                              .unwrap(),
                          vec!["mick".to_string(), "steve".to_string()])
                 .unwrap());

}
