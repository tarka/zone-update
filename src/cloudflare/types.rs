use serde::{Deserialize, Serialize};

use crate::RecordType;

#[derive(Debug, Deserialize)]
pub(crate) struct Response<T> {
    pub success: bool,
    pub result: T,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ZoneInfo {
    pub id: String,
    pub name: String,
    // ...
}

// {
//     "name": "example.com",
//     "ttl": 3600,
//     "type": "A",
//     "comment": "Domain verification record",
//     "content": "198.51.100.4",
//     "proxied": true,
//     "settings": {
//         "ipv4_only": true,
//         "ipv6_only": true
//     },
//     "tags": [
//         "owner:dns-team"
//     ],
//     "id": "023e105f4ecef8ad9ca31a8372d0c353",
//     "created_on": "2014-01-01T05:20:00.12345Z",
//     "meta": {},
//     "modified_on": "2014-01-01T05:20:00.12345Z",
//     "proxiable": true,
//     "comment_modified_on": "2024-01-01T05:20:00.12345Z",
//     "tags_modified_on": "2025-01-01T05:20:00.12345Z"
// }
#[derive(Deserialize, Debug, Clone)]
pub struct GetRecord<T>
{
    pub id: String,
    pub name: String,
    pub ttl: u32,
    #[serde(rename = "type")]
    pub rtype: RecordType,
    pub content: T,
}

pub type GetRecords<T> = Vec<GetRecord<T>>;


// {
//     "name": "example.com",
//     "ttl": 3600,
//     "type": "A",
//     "comment": "Domain verification record",
//     "content": "198.51.100.4",
//     "proxied": true
// }
#[derive(Serialize, Debug, Clone)]
pub struct CreateRecord<T> {
    pub name: String,
    pub ttl: u32,
    #[serde(rename = "type")]
    pub rtype: RecordType,
    pub content: T,
}
