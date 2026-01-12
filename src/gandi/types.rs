use serde::{Deserialize, Serialize};

// See https://api.gandi.net/docs/livedns/

// {
//   "object": "HTTPNotFound",
//   "cause": "Not Found",
//   "code": 404,
//   "message": "The resource could not be found."
// }
//
// Note: Not currently decoded, but body is logged as part of
// `check_error()`;
/// Minimal error response returned by the Gandi API.
#[allow(unused)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub object: String,
    pub cause: String,
    pub code: u32,
    pub message: String,
}

// [
//   {
//     "rrset_name": "@",
//     "rrset_ttl": 10800,
//     "rrset_type": "A",
//     "rrset_values": [
//       "192.0.2.1"
//     ],
//     "rrset_href": "https://api.test/v5/livedns/domains/example.com/records/%40/A"
//   },
// ]
/// Representation of a Gandi DNS record set.
#[derive(Serialize, Deserialize, Debug)]
pub struct Record<T>
{
    pub rrset_name: String,
    pub rrset_type: String,
    pub rrset_values: Vec<T>,
    pub rrset_href: String,
    pub rrset_ttl: Option<u32>,
}

// {
//   "rrset_values": [
//     "www.example.org"
//   ],
//   "rrset_ttl": 320
// }
/// Payload used to update a Gandi DNS record set.
#[derive(Serialize, Deserialize, Debug)]
pub struct RecordUpdate<T> {
    pub rrset_values: Vec<T>,
    pub rrset_ttl: Option<u32>,
}
