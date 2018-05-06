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
#[macro_use]
extern crate log;

mod repo;
mod imp;
mod web;
mod journal;

use futures::Future;
use actix::{msgs::StartActor, prelude::*};
use actix::actors::signal::DefaultSignalsHandler;
use actix_web::server;

use repo::Repo;
use imp::{Importer, ScanDir};
use journal::Journal;

struct ProgArgs {
    json_path: String,
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
            .arg(Arg::with_name("web")
                 .help("Start the web service")
                 .long("web")
                 .value_name("LISTEN_ADDR")
                 .default_value("localhost:8080"))
            .get_matches();

        Self {
            json_path: args.value_of("JSON_FILE").unwrap().to_string(),
            web: args.value_of("web").map(|v| v.to_string()),
        }
    }
}

fn main() {
    let sys = System::new("Bug Graph");
    let _journal = Arbiter::system_registry().get::<Journal>();
    let pargs = ProgArgs::parse();

    let repo_arb = Arbiter::new("repository");
    let imp_arb = Arbiter::new("importer");

    let _signals: Addr<Unsync, _> = DefaultSignalsHandler::default().start();
    {
        let json_path = pargs.json_path.clone();
        let imp_arb = imp_arb.clone();

        Arbiter::handle().spawn(repo_arb
            .send(StartActor::new(|_| Repo::default()))
            .then(move |repo| match repo {
                Ok(repo) => imp_arb.send(StartActor::new(move |_| Importer::new(repo))),
                Err(e) => panic!("Could not start repository: {}", e),
            })
            .then(|imp| match imp {
                Ok(imp) => imp.send(ScanDir { dir: json_path, ext: "json".to_string() }),
                Err(e) => panic!("Could not start importer: {}", e),
            })
            .map_err(|e| error!("Scan directory: {}", e)));
    }

    if let Some(ref addr) = pargs.web {
        if let Err(e) = server::new(|| web::new()).bind(addr) {
            error!("Failed to start web server on {}: {}", addr, e);
        }
    }

    sys.run();
}
