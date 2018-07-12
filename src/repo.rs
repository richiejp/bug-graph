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

use std::convert::Into;
use std::collections::{BTreeMap, HashMap};

use indradb::{Vertex, Type, EdgeKey, VertexQuery, Datastore, MemoryDatastore, Transaction};
use indradb::Result as IResult;
use actix::prelude::*;
use uuid::Uuid;

use protocol::{VertInfo, ResultMatrix, ResultInMatrix};

macro_rules! itype {
    ($vert_name:ident) => (
        Type(stringify!($vert_name).to_string());
    )
}

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

pub struct GetSetVerts(pub Option<Uuid>);

impl Message for GetSetVerts {
    type Result = Vec<(String, Uuid)>;
}

#[derive(Message)]
#[rtype(result = "Vec<(String, Uuid)>")]
pub struct Search(pub String);

#[derive(Message)]
#[rtype(ResultMatrix)]
pub struct GetResultMatrix(pub Uuid);

#[derive(Default)]
struct VertNameIndex {
    verts: BTreeMap<String, Uuid>,
    names: BTreeMap<Uuid, String>,
}

impl VertNameIndex {
    fn contains_name(&self, name: &str) -> bool {
        self.verts.contains_key(name)
    }

    fn insert<S: Into<String>>(&mut self, name: S, vert: Uuid) {
        let name = name.into();

        self.verts.insert(name.clone(), vert);
        self.names.insert(vert, name);
    }

    fn get_all(&self) -> Vec<(String, Uuid)> {
        self.verts.iter().map(|(name, uuid)| (name.clone(), *uuid)).collect()
    }

    fn get_name(&self, vert: &Uuid) -> Option<&String> {
        self.names.get(vert)
    }

    fn get_vert(&self, name: &str) -> Option<&Uuid> {
        self.verts.get(name)
    }

    fn search(&self, name: String) -> Vec<(String, Uuid)> {
        self.verts.range(name..).take(10)
            .map(|(name, uuid)| (name.clone(), *uuid)).collect()
    }
}

pub struct Repo {
    indradb: MemoryDatastore,
    id_indx: VertNameIndex,
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

    fn intern_name<T>(&mut self, t: &T, name_of: &Type, name: &str) -> Uuid
    where
        T: Transaction
    {
        if !self.id_indx.contains_name(name) {
            self.id_indx.insert(name, new_vert(t, name_of));
        }

        *self.id_indx.get_vert(name).unwrap()
    }

    fn intern_fq_name<T>(&mut self, t: &T, name_of: &Type, name: &str) -> Uuid
    where
        T: Transaction
    {
        let mut outer_cat: Option<Uuid> = None;

        for (i, c) in name.chars().enumerate() {
            if c == ':' {
                let category = self.intern_name(t, &SET_VT, &name[0 .. i]);
                if let Some(ref ocat) = outer_cat {
                    new_edge(t, &category, &ISIN_ET, ocat);
                }
                outer_cat = Some(category);
            }
        }

        let tvid = self.intern_name(t, name_of, name);
        if let Some(ref ocat) = outer_cat {
            new_edge(t, &tvid, &ISIN_ET, ocat);
        }

        tvid
    }

    fn get_adjacent<T: Transaction>(&self, t: &T, vert: Uuid) -> IResult<Vec<(String, Uuid)>> {
        let q = VertexQuery::Vertices { ids: vec![vert] };

        Ok(t.get_vertices(&q.inbound_edges(None, None, None, 1000).outbound_vertices(1000))?
           .iter()
           .filter_map(|v| self.id_indx.get_name(&v.id).and_then(|n| Some((n.clone(), v.id))))
           .collect()
        )
    }

    fn get_test_results<T: Transaction>(&self, t: &T, test: Uuid)
                                        -> IResult<Vec<(Uuid, bool)>> {
        let q = VertexQuery::Vertices { ids: vec![test] };

        let passed = t.get_vertices(&q.outbound_edges(Some(PASS_ET.clone()), None, None, 500)
                                    .inbound_vertices(500))?;
        let res = passed.into_iter().map(|v| (v.id, true));

        let q = VertexQuery::Vertices { ids: vec![test] };
        Ok(res.chain(t.get_vertices(&q.outbound_edges(Some(FAIL_ET.clone()), None, None, 500)
                                 .inbound_vertices(500))?
                     .into_iter()
                     .map(|v| (v.id, false))).collect())
    }

    fn get_test_result_props<T: Transaction>(&self, t: &T, test_result: Uuid)
                                               -> IResult<Vec<VertInfo>> {
        let q = (VertexQuery::Vertices { ids: vec![test_result] })
            .outbound_edges(Some(ISIN_ET.clone()), None, None, 1000)
            .inbound_vertices(1000);
        Ok(t.get_vertices(&q)?
           .iter()
           .filter_map(|v| {
               self.id_indx.get_name(&v.id).and_then(|n| Some(VertInfo(n.clone(), v.id)))
           })
           .collect())
    }

    fn get_outer_sets<T: Transaction>(&self, t: &T, ids: Vec<Uuid>) -> IResult<Vec<Vertex>> {
        let q = VertexQuery::Vertices { ids };
        t.get_vertices(&q.inbound_edges(Some(ISIN_ET.clone()), None, None, 100)
                       .outbound_vertices(100))
    }

    fn get_inner_tests<T: Transaction>(&self, t: &T, sets_or_tests: Vec<Uuid>)
                                       -> IResult<Vec<VertInfo>> {
        let q = VertexQuery::Vertices { ids: sets_or_tests };
        let mut depth = 0;
        let mut res: Vec<VertInfo> = Vec::new();
        let mut verts = t.get_vertices(&q)?;

        while res.len() < 100 && verts.len() > 0 && depth < 10 {
            {
                let tests = verts.iter()
                    .filter(|v| &v.t == &*TEST_VT)
                    .map(|v| {
                        VertInfo(self.id_indx.get_name(&v.id).unwrap_or(&String::new()).clone(),
                                 v.id)
                    });
                res.extend(tests);
            }

            let ids = verts.iter().filter(|v| &v.t == &*SET_VT).map(|sv| sv.id).collect();
            verts = self.get_outer_sets(t, ids)?;

            depth += 1;
        }

        Ok(res)
    }

}

impl Default for Repo {
    fn default() -> Self {
        let ds = MemoryDatastore::default();

        Repo {
            indradb: ds,
            id_indx: VertNameIndex::default(),
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
        let test = self.intern_fq_name(&t, &TEST_VT, &msg.test_fqn);
        let result = new_vert(&t, &TEST_RES_VT);
        new_edge(&t, &test, msg.status.into(), &result);

        for name in &msg.properties {
            let prop = self.intern_fq_name(&t, &SET_VT, &name);
            new_edge(&t, &result, &ISIN_ET, &prop);
        }

        MessageResult(result)
    }
}

impl Handler<GetSetVerts> for Repo {
    type Result = MessageResult<GetSetVerts>;

    fn handle(&mut self, msg: GetSetVerts, _: &mut Self::Context) -> Self::Result {
        MessageResult(
            if let Some(vert) = msg.0 {
                self.indradb.transaction()
                    .and_then(|t| self.get_adjacent(&t, vert))
                    .unwrap_or_else(|e| {
                        error!("Could not get vertices: {}", e);
                        Vec::default()
                    })
            } else {
                self.id_indx.get_all()
            }
        )
    }
}

impl Handler<Search> for Repo {
    type Result = MessageResult<Search>;

    fn handle(&mut self, msg: Search, _: &mut Self::Context) -> Self::Result {
        MessageResult(self.id_indx.search(msg.0))
    }
}

impl Handler<GetResultMatrix> for Repo {
    type Result = MessageResult<GetResultMatrix>;

    fn handle(&mut self, msg: GetResultMatrix, _ctx: &mut Self::Context) -> Self::Result {
        let t = self.indradb.transaction().unwrap();
        
        let tests = self.get_inner_tests(&t, vec![msg.0]).unwrap_or_else(|e| {
            error!("Failed to get inner tests: {}", e);
            Vec::default()
        });

        if tests.len() < 1 {
            debug!("No inner tests returned for {}", msg.0);
        }

        // Results keyed by build/product/test property
        let mut results = HashMap::<Uuid, (VertInfo, Vec<ResultInMatrix>)>::new();

        for (i, VertInfo(test_name, test)) in tests.iter().enumerate() {
            let results2 = self.get_test_results(&t, *test).unwrap_or_else(|e| {
                error!("Failed test results: {}", e);
                Vec::default()
            });

            if results2.len() < 1 {
                debug!("No test-results found for {} ({})", test_name, test);
            }

            let mut results3 = HashMap::<Uuid, (VertInfo, u32, u32)>::new();
            for (result2, status) in results2.into_iter() {
                let props = self.get_test_result_props(&t, result2).unwrap_or_else(|e| {
                    error!("Failed to get test result properties/sets: {}", e);
                    Vec::default()
                });

                if props.len() < 1 {
                    debug!("No test-properties found for {}'s test-result: {}",
                           test_name, result2);
                }

                for prop in props.into_iter() {
                    let result3 = results3.entry(prop.1).or_insert((prop, 0, 0));
                    if status {
                        result3.1 += 1;
                    } else {
                        result3.2 += 1;
                    }
                }
            }

            for (prop, (vinfo, passes, fails)) in results3.into_iter() {
                let result = results.entry(prop).or_insert_with(|| {
                    (vinfo, Vec::new())
                });

                result.1.push(ResultInMatrix {
                    test_case: i as u32,
                    passes,
                    fails,
                });
            }
        }
        
        MessageResult(ResultMatrix {
            test_cases: tests,
            results: results.into_iter().map(|(_, v)| (v.0, v.1)).collect(),
        })
    }
}
