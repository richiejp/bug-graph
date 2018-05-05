extern crate indradb;
extern crate clap;
extern crate serde_json;
extern crate uuid;
#[macro_use]
extern crate lazy_static;
extern crate futures;
#[macro_use]
extern crate actix;
extern crate actix_web;
extern crate log;

mod repo;
mod imp;
mod web;
mod journal;

use std::path::Path;
use std::iter::Iterator;

use actix::prelude::*;
use actix::actors::signal::DefaultSignalsHandler;
use repo::Repo;
use uuid::Uuid;

use imp::{Importer, ScanDir};

struct ProgArgs {
    json_path: String,
    dot_path: Option<String>,
    web: Option<String>,
}

impl ProgArgs {
    fn parse() -> Self {
        use clap::{App, Arg};

        let args = App::new("Bug Graph")
            .arg(Arg::with_name("JSON_FILE")
                 .help("Test results")
                 .required(true)
                 .index(1))
            .arg(Arg::with_name("DOT_FILE")
                 .help("Name of a dot file to dump the graph to")
                 .index(2))
            .arg(Arg::with_name("web")
                 .help("Start the web service")
                 .long("web")
                 .value_name("LISTEN_ADDR")
                 .default_value("localhost:8080"))
            .get_matches();

        Self {
            json_path: args.value_of("JSON_FILE").unwrap().to_string(),
            dot_path: args.value_of("DOT_FILE").map(|v| v.to_string()),
            web: args.value_of("web").map(|v| v.to_string()),
        }
    }
}

fn main() {
    let pargs = ProgArgs::parse();
    let sys = System::new("Bug Graph");
    let signals: Addr<Unsync, _> = DefaultSignalsHandler::default().start();

    let repo = Arbiter::start(|| Repo::default());
    let imp = {
        let repo = repo.clone();
        Arbiter::start(|| Importer { repo: repo })
    };

    if let Some(ref addr) = pargs.web {
        web::run(addr);
    }

    sys.run();
}
