
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    http::de_str,
    porkbun::Auth,
    RecordType
};

// This could be folded into the records below with #[serde(flatten)],
// but isn't worth it.
#[derive(Deserialize, Serialize, Debug)]
pub struct AuthOnly {
    pub secretapikey: String,
    pub apikey: String,
}

impl From<Auth> for AuthOnly {
    fn from(value: Auth) -> Self {
        Self {
            secretapikey: value.secret,
            apikey: value.key,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IdOnly {
    #[serde(deserialize_with = "de_str")]
    id: u64,
}

// {
// 	"secretapikey": "YOUR_SECRET_API_KEY",
// 	"apikey": "YOUR_API_KEY",
// 	"name": "www",
// 	"type": "A",
// 	"content": "1.1.1.1",
// 	"ttl": "600"
// }
#[derive(Deserialize, Serialize, Debug)]
pub struct CreateUpdate<T> {
    pub secretapikey: String,
    pub apikey: String,
    pub name: String,
    #[serde(rename = "type")]
    pub rtype: RecordType,
    pub content: T,
    pub ttl: u32,
}


// {
//     "status": "SUCCESS",
//     "records": [
// 	{
// 	    "id": "106926659",
// 	    "name": "www.borseth.ink",
// 	    "type": "A",
// 	    "content": "1.1.1.1",
// 	    "ttl": "600",
// 	    "prio": "0",
// 	    "notes": ""
// 	}
//     ]
// }
#[derive(Deserialize, Serialize, Debug)]
pub struct Record<T> {
    #[serde(deserialize_with = "de_str")]
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub rtype: RecordType,
    pub content: T,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Records<T> {
    pub records: Vec<Record<T>>
}
