use indradb::{Type, EdgeKey, Vertex, MemoryDatastore, MemoryTransaction};
use actix::prelude::*;
use std::collections::BTreeMap;
use uuid::Uuid;

macro_rules! itype {
    ($vert_name:ident) => (
        Type(stringify!($vert_name).to_string());
    )
}

pub static PASS: &str = "passed";
pub static FAIL: &str = "failed";
pub static ISIN: &str = "is_in";
pub static TEST: &str = "test";
pub static TEST_RES: &str = "result";
pub static SET: &str = "set";

lazy_static! {
    pub static ref PASS_ET: Type = itype!(passed);
    pub static ref FAIL_ET: Type = itype!(failed);
    pub static ref ISIN_ET: Type = itype!(is_in);

    pub static ref TEST_VT: Type = itype!(test);
    pub static ref TEST_RES_VT: Type = itype!(result);
    pub static ref SET_VT: Type = itype!(set);
}

#[derive(Message)]
#[rtype(result = "Uuid")]
pub struct InternName {
    pub stem: String,
    pub name: String,
}

impl InternName {
    fn new<S: Into<String>, N: Into<String>>(stem: S, name: N) -> Self {
        Self {
            stem: stem.into(),
            name: name.into(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "Uuid")]
pub struct InternTest(String);

impl InternTest {
    fn new<S: Into<String>>(fqn: S) -> Self {
        Self(fqn.into())
    }
}

#[derive(Message)]
#[rtype(result = "Uuid")]
pub struct NewVert(String);

impl NewVert {
    fn new<S: Into<String>>(vtype: S) -> Self {
        Self(vtype.into())
    }
}

#[derive(Message)]
#[rtype(result = "Uuid")]
pub struct NewEdge(Uuid, String, Uuid);

impl NewEdge {
    fn new<S: Into<String>>(from: Uuid, etype: S, to: Uuid) -> Self {
        Self(from, etype.into(), to)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<>>")]
pub struct NewEdge(Uuid, String, Uuid);

impl NewEdge {
    fn new<S: Into<String>>(from: Uuid, etype: S, to: Uuid) -> Self {
        Self(from, etype.into(), to)
    }
}

pub struct Repo {
    indradb: MemoryDatastore,
    t: MemoryTransaction,
    id_indx: BTreeMap<String, Uuid>,
}

impl Repo {
    fn intern_name<'a>(vtype: &Type, name: &str) -> &'a Uuid {
        let key = format!("{}|{}", vtype.0, name);
        self.id_indx.entry(key).or_insert_with(|| {
            self.t.create_vertex(vtype).unwrap()
        })
    }
}

impl Default for Repo {
    fn default() -> Self {
        let ds = MemoryDatastore::default();

        Repo {
            indradb: ds,
            t: ds.transaction().unwrap(),
            id_indx: BTreeMap::default(),
        }
    }
}

impl Actor for Repo {
    type Context = Context<Self>;
}

impl Handler<InternName> for Repo {
    type Result = Uuid;

    fn handle(&mut self, msg: InternName, _: &mut Context<Self>)
              -> Self::Result {
        self.internName(&Type(msg.stem), &msg.name);
    }
}

impl Handler<InternTest> for Repo {
    type Result = Uuid;

    fn handle(&mut self, msg: InternTest, _: &mut Context<Self>)
              -> Self::Result {
        let mut outer_cat: Option<Uuid> = None;

        for (i, c) in msg.0.chars().enumerate() {
            if c == ':' {
                let category = self.intern_name(&SET_VT, &msg.fqn[0 .. i]);
                if let Some(ocat) = outer_cat {
                    self.t.create_edge(
                        &EdgeKey::new(*category, ISIN_ET.clone(), ocat)
                    ).unwrap();
                }
                outer_cat = Some(*category);
            }
        }

        let tvid = self.intern_name(&TEST_VT, &msg.0);
        if let Some(ocat) = outer_cat {
            self.t.create_edge(&EdgeKey::new(tvid.clone(), ISIN_ET.clone(), ocat)).unwrap();
        }

        tvid
    }
}

impl Handler<NewVert> for Repo {
    type Result = Uuid;

    fn handle(&mut self, msg: NewVert, _: &mut Self::Context) -> Self::Result {
        self.t.create_vertex(&msg.0).unwrap()
    }
}

impl Handler<NewEdge> for Repo {
    type Result = Uuid;

    fn handle(&mut self, msg: NewEdge, _: &mut Self::Context) -> Self::Result {
        self.t.create_edge(&msg.0).unwrap()
    }
}
