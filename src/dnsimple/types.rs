
use std::{any::Any, marker::PhantomData, net::Ipv4Addr};

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::RecordType;


// {
//   "data": [
//     {
//       "id": 2602,
//       "email": "tarkasteve+dnsimple.sandbox@gmail.com",
//       "plan_identifier": "solo-v2-monthly",
//       "created_at": "2025-09-14T01:53:43Z",
//       "updated_at": "2025-09-14T01:56:14Z"
//     }
//   ]
// }
#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub id: u32,
    pub email: String,
    pub plan_identifier: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Accounts {
    #[serde(rename = "data")]
    pub accounts: Vec<Account>
}

// {
//   "data": [
//     {
//       "id": 3422640,
//       "zone_id": "testcondition.net",
//       "parent_id": null,
//       "name": "test",
//       "content": "1.2.3.4",
//       "ttl": 60,
//       "priority": null,
//       "type": "A",
//       "regions": [
//         "global"
//       ],
//       "system_record": false,
//       "created_at": "2025-09-20T01:10:32Z",
//       "updated_at": "2025-09-20T01:10:32Z"
//     }
//   ],
//   "pagination": {
//     "current_page": 1,
//     "per_page": 30,
//     "total_entries": 1,
//     "total_pages": 1
//   }
// }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRecord {
    pub id: u64,
    pub zone_id: String,
    pub name: String,
    pub content: Ipv4Addr,
    pub ttl: u32,
    #[serde(rename = "type")]
    pub rtype: RecordType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Records {
    #[serde(rename = "data")]
    pub records: Vec<GetRecord>
}



// {
//   "name": "",
//   "type": "MX",
//   "content": "mxa.example.com",
//   "ttl": 600,
//   "priority": 10,
//   "regions": ["SV1", "IAD"],
//   "integrated_zones": [1, 2, "dnsimple"]
// }
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRecord {
    pub name: String,
    #[serde(rename = "type")]
    pub rtype: RecordType,
    pub content: String,
    pub ttl: u32,
    // We can skip the rest; either not needed and/or unsupported on
    // some plans.
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRecord {
    pub content: String,
}
