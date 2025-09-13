// dns-edit: DNS provider update utilities
// Copyright (C) 2025 tarkasteve@gmail.com
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use serde::{Deserialize, Serialize};

// See https://api.gandi.net/docs/livedns/

// {
//   "object": "HTTPNotFound",
//   "cause": "Not Found",
//   "code": 404,
//   "message": "The resource could not be found."
// }
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
#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub rrset_name: String,
    pub rrset_type: String,
    pub rrset_values: Vec<String>,
    pub rrset_href: String,
    pub rrset_ttl: Option<u32>,
}

// {
//   "rrset_values": [
//     "www.example.org"
//   ],
//   "rrset_ttl": 320
// }
#[derive(Serialize, Deserialize, Debug)]
pub struct RecordUpdate {
    pub rrset_values: Vec<String>,
    pub rrset_ttl: Option<u32>,
}
