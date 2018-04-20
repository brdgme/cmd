use failure::Error;
use serde_json;

use std::ffi::OsString;
use std::io::{BufWriter, Write};
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
        let mut cmd = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        {
            let mut wr = cmd.stdin
                .as_mut()
                .ok_or(format_err!("failed to get stdin"))?;
            let mut bufwr = BufWriter::new(&mut wr);

            bufwr.write_all(serde_json::to_string(req)?.as_bytes())?;
            bufwr.flush()?;
        }

        let output = cmd.wait_with_output()?;

        match serde_json::from_slice(&output.stdout) {
            Ok(response) => Ok(response),
            Err(e) => {
                println!("{}", String::from_utf8(output.stdout).unwrap());
                panic!(e.to_string());
            }
        }
    }
}
