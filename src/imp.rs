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

use std::collections::VecDeque;
use std::fs::{self, File};
use std::ffi::OsString;
use std::io::prelude::*;
use std::path::Path;

use actix::dev::*;
use futures::Future;

use repo::{Repo, NewResult, TestStatus};

#[derive(Message)] pub struct ScanDir {
    pub dir: String,
    pub ext: String,
}
#[derive(Message)] pub struct Import(String);

pub struct Importer {
    repo: Addr<Syn, Repo>,
}

impl Importer {
    pub fn new(repo: Addr<Syn, Repo>) -> Importer {
        Importer {
            repo,
        }
    }

    fn read_file<P: AsRef<Path>>(path: P) -> String {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        contents
    }

    fn read_files(dir: String, ext: String) -> impl Iterator<Item = String> {
        let ext = OsString::from(ext);
        fs::read_dir(dir).unwrap().filter_map(|ent| {
            ent.ok()
        }).filter(|ent| {
            ent.file_type().map(|e| e.is_file()).unwrap_or(false)
        }).filter_map(move |ent| {
            let fp = ent.path();
            if fp.extension().map_or(false, |e| e == ext) {
                info!("Reading file: {}", fp.display());
                Some(Importer::read_file(fp))
            } else {
                info!("Ignoring file: {}", fp.display());
                None
            }
        })
    }
}

impl Actor for Importer {
    type Context = Context<Self>;
}

impl Handler<ScanDir> for Importer {
    type Result = ();

    fn handle(&mut self, msg: ScanDir, ctx: &mut Self::Context) {
        info!("Scanning directory: {}", &msg.dir);

        for json in Importer::read_files(msg.dir, msg.ext) {
            ctx.notify(Import(json));
        }
    }
}

impl Handler<Import> for Importer {
    type Result = ();

    fn handle(&mut self, msg: Import, _ctx: &mut Self::Context) {
        use serde_json::{self, *};

        let mut v: Value = serde_json::from_str(&msg.0).unwrap();
        let env_props = {
            let mut env = v["environment"].as_object_mut().unwrap();
            let product = format!("environment:product:{}:{}",
                                  env.remove("product").unwrap().as_str().unwrap(),
                                  env.remove("revision").unwrap().as_str().unwrap());
            let mut props = vec![product];
            for (key, val) in env.iter() {
                props.push(format!("environment:{}:{}", key, val));
            }
            props
        };

        let mut reqs: VecDeque<Request<Syn, Repo, NewResult>> = VecDeque::with_capacity(8);
        for r in v["results"].as_array().unwrap() {
            let mut props = env_props.clone();

            for (key, val) in r["test"].as_object().unwrap() {
                if key != "log" && key != "duration" {
                    props.push(format!("test:{}:{}", key, val));
                }
            }

            let status = if r["status"] == "pass" {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            };

            reqs.push_back(self.repo.send(NewResult {
                test_fqn: r["test_fqn"].as_str().unwrap().to_owned(),
                status,
                properties: props,
            }));

            if reqs.len() > 8 {
                if let Err(e) = reqs.pop_front().wait() {
                    error!("Aborting import; Repository returned error: {}", e);
                    break;
                }
            }
        }
    }
}
