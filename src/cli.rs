use serde::{Serialize, Deserialize};
use serde_json;

use brdgme_game::{Gamer, Log};
use brdgme_markup::Node;

use std::fmt::Debug;
use std::io::{Read, Write};

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
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
    content: Vec<Node>,
    at: String,
    public: bool,
    to: Vec<usize>,
}

impl CliLog {
    fn from_log(log: &Log) -> CliLog {
        CliLog {
            content: log.content.clone(),
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
struct Rendering {
    public: Vec<Node>,
    private: Vec<Vec<Node>>,
}

impl Rendering {
    fn from_gamer<T: Gamer>(gamer: &T) -> Rendering {
        use brdgme_game::Renderer;

        Rendering {
            public: gamer.pub_state(None).render(),
            private: vec![],
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum Response<T: Gamer + Debug + Clone + Serialize + Deserialize> {
    Game {
        game: T,
        logs: Vec<CliLog>,
        rendering: Rendering,
    },
    UserError { message: String },
    SystemError { message: String },
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
                                        Ok(Request::New { players }) => {
                                            match T::new(players) {
                                                Ok((game, logs)) => {
                                                    Response::Game {
                                                        rendering: Rendering::from_gamer(&game),
                                                        game: game,
                                                        logs: CliLog::from_logs(&logs),
                                                    }
                                                }
                                                Err(e) => {
                                                    Response::UserError { message: e.to_string() }
                                                }
                                            }
                                        }
                                        _ => panic!("egg"),
                                    })
                     .unwrap())
            .unwrap();
}
