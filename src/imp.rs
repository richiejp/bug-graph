use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;
use actix::prelude::*;

use repo::{InternName, InternTest, NewVert, NewEdge,
           SET, TEST_RES, ISIN, PASS, FAIL};

#[derive(Message)] pub struct ScanDir {
    dir: String,
    ext: String,
}
#[derive(Message)] pub struct Import(String);

pub struct Importer {
    repo: Addr<Syn, Repo>,
}

impl Importer {
    fn read_file<P: AsRef<Path>>(path: P) -> String {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        contents
    }

    fn read_files<'a>(dir: &str, ext: &'a str) -> impl Iterator<Item = String> {
        fs::read_dir(dir).unwrap().filter_map(|ent| {
            ent.ok()
        }).filter(|ent| {
            ent.file_type().map(|e| e.is_file()).unwrap_or(false)
        }).filter_map(move |ent| {
            let fp = ent.path();
            if fp.extension().map_or(false, |e| e == ext) {
                Some(read_file(fp))
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
        for json in self.read_files(msg.dir, msg.ext) {
            ctx.notify(Import(json));
        }
    }
}

impl Handler<Import> for Importer {
    type Result = ();

    fn handle(&mut self, msg: Import, ctx: &mut Self::Context) {
        use serde_json::{self, *};

        let v: Value = serde_json::from_str(tjson).unwrap();
        let env = v["environment"].as_object().unwrap();
        let (prd, rev) =
            self.repo.send(InternName::new(SET, env["product"].as_str().unwrap()))
            .join(self.repo.send(InternName::new(SET, env["revision"].as_str().unwrap())))
            .wait().unwrap();

        for r in v["results"].as_array().unwrap() {
            let vids =
                self.repo.send(NewVert::new(TEST_RES))
                .join(self.repo.send(InternTest::new(r["test_fqn"].as_str().unwrap())));

            let et = if r["status"] == "pass" {
                PASS
            } else {
                FAIL
            };

            ctx.spawn(vids.into_actor(self).and_then(|vids, act, ctx| {
                let (trvid, test_vid) = vids;
                self.repo.send(NewEdge::new(*test_vid, et, trvid))
                    .join3(
                        self.repo.send(NewEdge::new(*test_vid, ISIN, prd)),
                        self.repo.send(NewEdge::new(*test_vid, ISIN, rev)))
            }).drop_err().map(|_| ()));
        }
    }
}
