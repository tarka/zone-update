use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Domain {
    pub id: u32,
    pub name: String,
}
