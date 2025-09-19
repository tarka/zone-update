
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};




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

