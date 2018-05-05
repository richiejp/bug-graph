use indradb::*;
use actix::prelude::*;

pub struct Graph {
    ds: MemoryDatastore,
    t: MemoryTransaction,
}

