use std::collections::{HashMap, BTreeMap};
use uuid::Uuid;
use actix::prelude::*;

pub struct IndexKey<'a, 'b> {
    pub set_name: &'a str,
    pub key: &'b [u8],
}

impl<'a, 'b> IndexKey<'a, 'b> {
    pub fn new(set_name: &'a str, key: &'b [u8]) -> Self {
        IndexKey {
            set_name: set_name,
            key: key,
        }
    }
}

pub struct InternVert(indradb::Type, String);

impl Message for InternVert {
    type Result = Uuid;
}

type IndexRequest = Request<

pub struct Index {
    map: HashMap<String, BTreeMap<Vec<u8>, Uuid>>,
}

impl Handler<InternVert> for Index {
    type Result = Uuid;

    fn handle(&mut self, msg: InternVert, ctx: &mut Context<Self>) -> Self::Result {
        
    }
}

impl<'a> Index {
    pub fn new() -> Self {
        Index {
            map: HashMap::default(),
        }
    }

    pub fn get_or_insert_with<F: FnOnce() -> Uuid>(&'a mut self,
                                                       key: &IndexKey,
                                                       default: F)
                                                       -> &'a Uuid {
        if !self.map.contains_key(key.set_name) {
            self.map.insert(key.set_name.to_string(), BTreeMap::default());
        }
        let inner = self.map.get_mut(key.set_name).unwrap();

        if !inner.contains_key(key.key) {
            inner.insert(key.key.to_vec(), default());
        }
        inner.get(key.key).unwrap()
    }

    #[cfg(test)]
    pub fn get(&self, key: &IndexKey) -> Option<&Uuid> {
        self.map.get(key.set_name).and_then(|inner| {
            inner.get(key.key)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn add_get() {
        let set_name = "field";
        let keystr = "akey";
        let key = IndexKey::new(set_name, keystr.as_bytes());
        let val = Uuid::new_v4();
        let mut indx = Index::new();

        assert_eq!(indx.get_or_insert_with(&key, || val.clone()), &val);
        assert_eq!(indx.get(&key).unwrap(), &val);
    }
}
