use serde::{Deserialize, Serialize};

use crate::RecordType;


#[derive(Serialize, Deserialize, Debug)]
pub struct Domain {
    pub id: u32,
    pub name: String,
}

// {
//   "totalRecords": 3,
//   "totalPages": 1,
//   "data": [
//     {
//       "source": 1,
//       "ttl": 1800,
//       "gtdLocation": "DEFAULT",
//       "sourceId": 1119443,
//       "failover": false,
//       "monitor": false,
//       "hardLink": false,
//       "dynamicDns": false,
//       "failed": false,
//       "name": "ns1",
//       "value": "208.94.148.2",
//       "id": 66813434,
//       "type": "A"
//     },
//   ]
// }
#[derive(Serialize, Deserialize, Debug)]
pub struct Record<T> {
    pub id: u32,
    pub name: String,
    pub value: T,
    #[serde(rename = "type")]
    pub rtype: RecordType,
    #[serde(rename = "sourceId")]
    pub source_id: u32,
    pub ttl: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Records<T>
{
    #[serde(rename = "data")]
    pub records: Vec<Record<T>>
}
