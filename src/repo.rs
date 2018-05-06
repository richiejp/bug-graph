use std::convert::Into;

use indradb::{Type, EdgeKey, Datastore, MemoryDatastore, Transaction};
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
    id_indx: VertIndex,
}

fn new_edge<T: Transaction>(t: &T, egress: &Uuid, etype: &Type, ingress: &Uuid) {
    t.create_edge(
        &EdgeKey::new(*egress, etype.clone(), *ingress)
    ).unwrap();
}

fn new_vert<T: Transaction>(t: &T, vtype: &Type) -> Uuid {
    t.create_vertex_from_type(vtype.clone()).unwrap()
}

impl Repo {

    fn intern_name<'a, T>(&'a mut self, t: &T, name_of: &Type, name: &str) -> &'a Uuid
    where
        T: Transaction
    {
        if !self.id_indx.contains_key(name) {
            self.id_indx.insert(name.to_owned(), new_vert(t, name_of));
        }

        self.id_indx.get(name).unwrap()
    }

    fn intern_fq_name<'a, T>(&'a mut self, t: &T, name_of: &Type, name: &str) -> &'a Uuid
    where
        T: Transaction
    {
        let mut outer_cat: Option<Uuid> = None;

        for (i, c) in name.chars().enumerate() {
            if c == ':' {
                let category = self.intern_name(t, &SET_VT, &name[0 .. i]);
                if let Some(ref ocat) = outer_cat {
                    new_edge(t, category, &ISIN_ET, ocat);
                }
                outer_cat = Some(*category);
            }
        }

        let tvid = self.intern_name(t, name_of, name);
        if let Some(ref ocat) = outer_cat {
            new_edge(t, tvid, &ISIN_ET, ocat);
        }

        tvid
    }
}

impl Default for Repo {
    fn default() -> Self {
        let ds = MemoryDatastore::default();

        Repo {
            indradb: ds,
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
        let t = self.indradb.transaction().unwrap();
        let test = *self.intern_fq_name(&t, &TEST_VT, &msg.test_fqn);
        let result = new_vert(&t, &TEST_RES_VT);
        new_edge(&t, &test, msg.status.into(), &result);

        for name in msg.properties.iter() {
            let prop = self.intern_fq_name(&t, &SET_VT, &name);
            new_edge(&t, &test, &ISIN_ET, prop);
        }

        info!("Added test result for {}", &msg.test_fqn);
        MessageResult(result)
    }
}
