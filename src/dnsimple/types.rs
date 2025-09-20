
use std::net::Ipv4Addr;

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
#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
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
    pub records: Vec<Record>
}
