extern crate brdgme_markup;

use brdgme_markup::ansi;

fn main() {
    println!("{}",
             ansi("blah {{#b}}blah{{/b}} blah {{#bg red}}{{#fg \
                                  green}}CHRISTMAS{{/fg}}{{/bg}} {{player 0}} {{player 1}}",
                  vec!["mick".to_string(), "steve".to_string()])
                 .unwrap());

}
