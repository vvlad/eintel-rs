extern crate rusqlite;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct System {
    pub id: String,
    pub name: String,
    pub constelation: String,
    pub region: String
}


impl <'a, 'stmt>From<&'stmt rusqlite::Row<'a, 'stmt>> for System {
    fn from(row: &'stmt rusqlite::Row<'a, 'stmt>) -> Self {
        System{
            id: row.get(0),
            name: row.get(1),
            constelation: row.get(2),
            region: row.get(3)
        }
    }
}

impl System {
    pub fn requires_fields() -> Vec<&'static str> {
        vec![
            "id", 
            "name", 
            "constelation", 
            "region"
        ]
    }
}
