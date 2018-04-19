use failure::Error;
use serde_json;

use std::ffi::OsString;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::{Command, Stdio};

use api::{Request, Response};
use requester::Requester;

pub struct LocalRequester {
    path: OsString,
}

impl LocalRequester {
    pub fn new<I: Into<OsString>>(path: I) -> Self {
        LocalRequester { path: path.into() }
    }
}

impl Requester for LocalRequester {
    fn request(&mut self, req: &Request) -> Result<Response, Error> {
        let cmd = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut wr = cmd.stdin.unwrap();
        let mut bufwr = BufWriter::new(&mut wr);

        let mut rd = cmd.stdout.unwrap();
        let mut bufrd = BufReader::new(&mut rd);

        bufwr.write_all(serde_json::to_string(req)?.as_bytes())?;
        bufwr.flush()?;
        let mut output: Vec<u8> = vec![];
        bufrd.read_to_end(&mut output)?;
        match serde_json::from_slice(&output) {
            Ok(response) => Ok(response),
            Err(e) => {
                println!("{}", String::from_utf8(output).unwrap());
                panic!(e.to_string());
            }
        }
    }
}
