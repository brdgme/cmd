use failure::Error;

use api::{Request, Response};

pub mod gamer;

pub trait Requester {
    fn request(&mut self, req: &Request) -> Result<Response, Error>;
}
