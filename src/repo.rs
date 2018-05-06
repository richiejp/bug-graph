use std::convert::Into;

use indradb::{Type, EdgeKey, Datastore, MemoryDatastore, Transaction, MemoryTransaction};
use actix::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

macro_rules! itype {
    ($vert_name:ident) => (
        Type(stringify!($vert_name).to_string());
    )
}

// pub static PASS: &str = "passed";
// pub static FAIL: &str = "failed";
// pub static ISIN: &str = "is_in";
// pub static TEST: &str = "test";
// pub static TEST_RES: &str = "result";
// pub static SET: &str = "set";

lazy_static! {
    pub static ref PASS_ET: Type = itype!(passed);
    pub static ref FAIL_ET: Type = itype!(failed);
    pub static ref ISIN_ET: Type = itype!(is_in);

    pub static ref TEST_VT: Type = itype!(test);
    pub static ref TEST_RES_VT: Type = itype!(result);
    pub static ref SET_VT: Type = itype!(set);
}

#[derive(Message)]
#[rtype(Uuid)]
pub struct InternName {
    pub stem: String,
    pub name: String,
}

impl InternName {
    pub fn new<S: Into<String>, N: Into<String>>(stem: S, name: N) -> Self {
        Self {
            stem: stem.into(),
            name: name.into(),
        }
    }
}

#[derive(Message)]
#[rtype(Uuid)]
pub struct InternTest(String);

impl InternTest {
    pub fn new<S: Into<String>>(fqn: S) -> Self {
        InternTest(fqn.into())
    }
}

#[derive(Message)]
#[rtype(Uuid)]
pub struct NewVert(String);

impl NewVert {
    pub fn new<S: Into<String>>(vtype: S) -> Self {
        NewVert(vtype.into())
    }
}

#[derive(Message)]
#[rtype(bool)]
pub struct NewEdge(Uuid, String, Uuid);

impl NewEdge {
    pub fn new<S: Into<String>>(from: Uuid, etype: S, to: Uuid) -> Self {
        NewEdge(from, etype.into(), to)
    }
}

pub enum TestStatus {
    Pass,
    Fail
}

impl Into<&'static Type> for TestStatus {
    fn into(self) -> &'static Type {
        match self {
            TestStatus::Pass => &PASS_ET,
            TestStatus::Fail => &FAIL_ET,
        }
    }
}

#[derive(Message)]
#[rtype(Uuid)]
pub struct NewResult {
    pub test_fqn: String,
    pub status: TestStatus,
    pub properties: Vec<String>
}

type VertIndex = HashMap<String, Uuid>;

pub struct Repo {
    indradb: MemoryDatastore,
    t: MemoryTransaction,
    id_indx: VertIndex,
}

impl Repo {

    fn new_edge<'a>(&'a self, egress: &Uuid, etype: &Type, ingress: &Uuid) {
        self.t.create_edge(
            &EdgeKey::new(egress.clone(), etype.clone(), ingress.clone())
        ).unwrap();
    }

    fn intern_name<'a>(&'a mut self, name_of: &Type, name: &str) -> &'a Uuid {
        if !self.id_indx.contains_key(name) {
            self.id_indx.insert(name.to_owned(),
                                self.t.create_vertex(name_of).unwrap());
        }

        self.id_indx.get(name).unwrap()
    }

    fn intern_fq_name<'a>(&'a mut self, name_of: &Type, name: &str) -> &'a Uuid {
        let mut outer_cat: Option<Uuid> = None;

        for (i, c) in name.chars().enumerate() {
            if c == ':' {
                let category = self.intern_name(&SET_VT, &name[0 .. i]);
                if let Some(ref ocat) = outer_cat {
                    self.new_edge(category, &ISIN_ET, ocat);
                }
                outer_cat = Some(*category);
            }
        }

        let tvid = self.intern_name(name_of, name);
        if let Some(ref ocat) = outer_cat {
            self.new_edge(tvid, &ISIN_ET, ocat);
        }

        tvid
    }
}

impl Default for Repo {
    fn default() -> Self {
        let ds = MemoryDatastore::default();

        Repo {
            indradb: ds,
            t: ds.transaction().unwrap(),
            id_indx: HashMap::default(),
        }
    }
}

impl Actor for Repo {
    type Context = Context<Self>;
}

impl Handler<NewResult> for Repo {
    type Result = MessageResult<NewResult>;

    fn handle(&mut self, msg: NewResult, _: &mut Self::Context) -> Self::Result {
        let test = self.intern_fq_name(&TEST_VT, &msg.test_fqn);
        let result = self.t.create_vertex(&TEST_RES_VT).unwrap();
        self.new_edge(test, msg.status.into(), &result);

        for name in msg.properties.iter() {
            let prop = self.intern_fq_name(&SET_VT, &name);
            self.new_edge(test, &ISIN_ET, prop);
        }

        MessageResult(result)
    }
}

impl Handler<InternName> for Repo {
    type Result = MessageResult<InternName>;

    fn handle(&mut self, msg: InternName, _: &mut Context<Self>)
              -> Self::Result {
        MessageResult(*self.intern_name(&Type(msg.stem), &msg.name))
    }
}

impl Handler<InternTest> for Repo {
    type Result = MessageResult<InternTest>;

    fn handle(&mut self, msg: InternTest, _: &mut Context<Self>)
              -> Self::Result {

        MessageResult(*self.intern_fq_name(&TEST_VT, &msg.0))
    }
}

impl Handler<NewVert> for Repo {
    type Result = MessageResult<NewVert>;

    fn handle(&mut self, msg: NewVert, _: &mut Self::Context) -> Self::Result {
        MessageResult(self.t.create_vertex(&Type(msg.0)).unwrap())
    }
}

impl Handler<NewEdge> for Repo {
    type Result = bool;

    fn handle(&mut self, msg: NewEdge, _: &mut Self::Context) -> Self::Result {
        self.t.create_edge(&EdgeKey::new(msg.0, Type(msg.1), msg.2)).unwrap()
    }
}
