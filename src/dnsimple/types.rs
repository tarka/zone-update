
use serde::{Deserialize, Serialize};

// {
//   "data": {
//     "user": {
//       "id": 2471,
//       "email": "tarkasteve+dnsimple.sandbox@gmail.com",
//       "created_at": "2025-09-14T01:53:43Z",
//       "updated_at": "2025-09-14T01:53:43Z"
//     },
//     "account": null
//   }
// }
#[derive(Serialize, Deserialize, Debug)]
pub struct WhoAmI {
    
}
