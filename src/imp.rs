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
            repo: repo,
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
                Some(Importer::read_file(fp))
            } else {
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
        for json in Importer::read_files(msg.dir, msg.ext) {
            ctx.notify(Import(json));
        }
    }
}

impl Handler<Import> for Importer {
    type Result = ();

    fn handle(&mut self, msg: Import, ctx: &mut Self::Context) {
        use serde_json::{self, *};

        let mut v: Value = serde_json::from_str(&msg.0).unwrap();
        let mut env = v["environment"].as_object_mut().unwrap();
        let product = format!("environment:product:{}:{}",
                              env.remove("product").unwrap().as_str().unwrap(),
                              env.remove("revision").unwrap().as_str().unwrap());
        let mut env_props = vec![product];

        for (key, val) in env.iter() {
            env_props.push(format!("environment:{}:{}", key, val));
        }

        for r in v["results"].as_array().unwrap() {
            let mut props = env_props.clone();

            for (key, val) in r["test"].as_object().unwrap() {
                props.push(format!("test:{}:{}", key, val));
            }

            let status = if r["status"] == "pass" {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            };

            let req: Request<Syn, Repo, NewResult> = self.repo.send(NewResult {
                test_fqn: r["test_fqn"].as_str().unwrap().to_owned(),
                status: status,
                properties: props,
            });
            ctx.spawn(req.then(|res| {
                if let Err(MailboxError::Timeout) = res {
                    warn!("Repository backpressure; dropping test result");
                }
                Ok(())
            }).into_actor(self));
        }
    }
}
