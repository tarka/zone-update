use serde::{Deserialize, Serialize};

use crate::RecordType;


// {
//     "type": "A",
//     "name": "www",
//     "data": "162.10.66.0",
//     "priority": null,
//     "port": null,
//     "ttl": 1800,
//     "weight": null,
//     "flags": null,
//     "tag": null
// }
#[derive(Serialize, Debug, Clone)]
pub(crate) struct CreateUpdate<T> {
    #[serde(rename = "type")]
    pub rtype: RecordType,
    /// Short name e.g. www
    pub name: String,
    pub ttl: u32,
    pub data: T,

}

// {
//   "domain_records": [
//     {
//       "id": 28448432,
//       "type": "A",
//       "name": "@",
//       "data": "1.2.3.4",
//       "priority": null,
//       "port": null,
//       "ttl": 1800,
//       "weight": null,
//       "flags": null,
//       "tag": null
//     }
//   ],
//   "links": {},
//   "meta": {
//     "total": 4
//   }
// }
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Records<T> {
    pub domain_records: Vec<Record<T>>,

}

#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Record<T> {
    pub id: u64,
    #[serde(rename = "type")]
    pub rtype: RecordType,
    /// Short name e.g. www
    pub name: String,
    pub ttl: u32,
    pub data: T,

}
