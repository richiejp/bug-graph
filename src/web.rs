use actix::prelude::*;
use actix_web::{server, App, HttpRequest};

fn index(_req: HttpRequest) -> &'static str {
    "Hello world!"
}

pub fn new() -> App<()>
{
    App::new().resource("/", |r| r.f(index))
}
