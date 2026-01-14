
use serde::{Deserialize, Serialize};

use crate::RecordType;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct List<T> {
    pub(crate) data: Vec<T>,
}

// {
//   "axfr_ips": [],
//   "description": null,
//   "domain": "example.org",
//   "expire_sec": 300,
//   "id": 1234,
//   "master_ips": [],
//   "refresh_sec": 300,
//   "retry_sec": 300,
//   "soa_email": "admin@example.org",
//   "status": "active",
//   "tags": [
//     "example tag",
//     "another example"
//   ],
//   "ttl_sec": 300,
//   "type": "master"
// }
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Domain {
    pub(crate) id: u64,
    pub(crate) domain: String,
}


// {
//   "data": [
//     {
//       "created": "2018-01-01T00:01:01",
//       "id": 123456,
//       "name": "test",
//       "port": 80,
//       "priority": 50,
//       "protocol": null,
//       "service": null,
//       "tag": null,
//       "target": "192.0.2.0",
//       "ttl_sec": 604800,
//       "type": "A",
//       "updated": "2018-01-01T00:01:01",
//       "weight": 50
//     }
//   ],
//   "page": 1,
//   "pages": 1,
//   "results": 1
// }
// #[derive(Deserialize, Debug, Clone)]
// pub(crate) struct Record<T> {
//     pub(crate) id: u64,
//     pub(crate) name: String,
//     pub(crate) target: T,
//     #[serde(rename = "type")]
//     pub(crate) rtype: RecordType,
// }

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Record<T> {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) target: T,
    #[serde(rename = "type")]
    pub(crate) rtype: RecordType,
}

// #[derive(Deserialize, Debug, Clone)]
// #[serde(tag = "rtype", content = "target")]
// pub(crate) enum TaggedTarget {
//     A(Ipv4Addr),
//     AAAA(Ipv6Addr),
//     TXT(String),
//     Other,
// }

// // There's no simple method to filter returned values by RecordType in
// // the Linode API, so we accept raw string `target` here and
// // deserialise in `get_upstream_record()` once RecordType is known.
// #[derive(Deserialize, Debug, Clone)]
// pub(crate) struct RawRecord {
//     pub(crate) id: u64,
//     pub(crate) name: String,
//     pub(crate) target: String,
//     #[serde(rename = "type")]
//     pub(crate) rtype: RecordType,
// }

// {
//   "created": "2018-01-01T00:01:01",
//   "id": 123456,
//   "name": "test",
//   "port": 80,
//   "priority": 50,
//   "protocol": null,
//   "service": null,
//   "tag": null,
//   "target": "192.0.2.0",
//   "ttl_sec": 604800,
//   "type": "A",
//   "updated": "2018-01-01T00:01:01",
//   "weight": 50
// }
#[derive(Serialize, Debug, Clone)]
pub(crate) struct CreateUpdate<T> {
    pub(crate) name: String,
    pub(crate) target: T,
    pub(crate) ttl_sec: u64,
    #[serde(rename = "type")]
    pub(crate) rtype: RecordType,
}
