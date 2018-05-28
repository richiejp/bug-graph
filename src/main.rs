// Copyright (C) 2018 Richard Palethorpe <richiejp@f-m.fm>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

extern crate indradb;
extern crate clap;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate uuid;
#[macro_use]
extern crate lazy_static;
extern crate futures;
#[macro_use]
extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate log;
extern crate failure;

mod repo;
mod imp;
mod web;
mod journal;
mod protocol;

use futures::Future;
use actix::{msgs::{Execute, StartActor}, prelude::*};
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

fn start_web_server(web_arb: Addr<Syn, Arbiter>, repo: Addr<Syn, Repo>, url: String) {
    web_arb.do_send(Execute::new(move || -> Result<(), ()> {
        match server::new(move || web::new(repo.clone())).bind(url.clone()) {
            Err(e) => error!("Failed to bind web server to {}: {}", url, e),
            Ok(srv) => {
                srv.start();
            }
        };
        Ok(())
    }));
}

fn main() {
    let sys = System::new("Bug Graph");
    let journal = Arbiter::system_registry().get::<Journal>();
    journal.do_send(journal::Log { src: "main".into(),
                                   msg: "Bug Graph 0.1.0".into() });

    let pargs = ProgArgs::parse();
    let repo_arb = Arbiter::new("repository");
    let imp_arb = Arbiter::new("importer");
    let web_arb = Arbiter::new("web");

    {
        let json_path = pargs.json_path.clone();
        let imp_arb = imp_arb.clone();

        Arbiter::handle().spawn(repo_arb
            .send(StartActor::new(|_| Repo::default()))
            .then(move |repo| match repo {
                Ok(repo) => {
                    start_web_server(web_arb, repo.clone(), pargs.web.unwrap().into());
                    imp_arb.send(StartActor::new(move |_| Importer::new(repo)))
                },
                Err(e) => panic!("Could not start repository: {}", e),
            })
            .then(|imp| match imp {
                Ok(imp) => imp.send(ScanDir { dir: json_path, ext: "json".to_string() }),
                Err(e) => panic!("Could not start importer: {}", e),
            })
            .map_err(|e| error!("Scan directory: {}", e)));
    }

    journal::init();
    sys.run();
}
