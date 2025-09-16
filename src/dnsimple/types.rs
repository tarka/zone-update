
use chrono::NaiveDateTime;
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
    id: u64,
    email: String,
    plan_identifier: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Accounts {
    accounts: Vec<Account>
}

