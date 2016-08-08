extern crate brdgme_color;

fn main() {
    for (i, c) in brdgme_color::player_colors().iter().enumerate() {
        println!("{}Player {}",
                 brdgme_color::Style { fg: c.mono().inv(), bg: *c, bold: true }.ansi(),
                 i+1,
        );
    }
    for (i, c) in brdgme_color::player_colors().iter().enumerate() {
        println!("{}Player {}",
                 brdgme_color::Style { fg: *c, bg: brdgme_color::WHITE, bold: true }.ansi(),
                 i+1,
        );
    }
}
