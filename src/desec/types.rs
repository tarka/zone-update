use serde::{Deserialize, Serialize};

use crate::RecordType;


// {
//   "created": "2026-01-10T00:08:04.260224Z",
//   "domain": "haltcondition.dedyn.io",
//   "subname": "",
//   "name": "haltcondition.dedyn.io.",
//   "records": [
//     "170.64.213.116"
//   ],
//   "ttl": 3600,
//   "type": "A",
//   "touched": "2026-01-10T00:08:04.265704Z"
// }
#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RRSet<T>
{
    pub domain: String,
    /// Full name e.g. www.example.com
    pub name: String,
    /// Short name e.g. www
    pub subname: String,
    #[serde(rename = "type")]
    pub rtype: RecordType,
    pub ttl: u32,
    pub records: Vec<T>,
}

// Just a subset of RRSet
#[derive(Serialize, Debug, Clone)]
pub(crate) struct CreateUpdateRRSet<T> {
    pub subname: String,
    #[serde(rename = "type")]
    pub rtype: RecordType,
    pub ttl: u32,
    pub records: Vec<T>,
}
