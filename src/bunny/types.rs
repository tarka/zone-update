use serde::{Deserialize, Deserializer, Serialize};

use crate::{errors::Error, RecordType};


#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ZoneInfo {
    pub id: u64,
    pub domain: String,
    // ...
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ZoneList {
    pub items: Vec<ZoneInfo>,
}

// {
//   "Id": 12681448,
//   "Type": 0,
//   "Ttl": 300,
//   "Value": "170.64.213.116",
//   "Name": "www",
//   "Weight": 100,
//   "Priority": 0,
//   "Port": 0,
//   "Flags": 0,
//   "Tag": "",
//   "Accelerated": false,
//   "AcceleratedPullZoneId": 0,
//   "LinkName": "",
//   "IPGeoLocationInfo": {
//     "CountryCode": "AU",
//     "Country": "Australia",
//     "ASN": 14061,
//     "OrganizationName": "Digital Ocean",
//     "City": "Sydney"
//   },
//   "GeolocationInfo": null,
//   "MonitorStatus": 0,
//   "MonitorType": 0,
//   "GeolocationLatitude": 0.0,
//   "GeolocationLongitude": 0.0,
//   "EnviromentalVariables": [],
//   "LatencyZone": null,
//   "SmartRoutingType": 0,
//   "Disabled": false,
//   "Comment": null,
//   "AutoSslIssuance": true
// }
#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Record<T> {
    pub id: u64,
    #[serde(rename = "Type", deserialize_with = "de_recordtype")]
    pub rtype: RecordType,
    pub value: T,
    pub name: String,
    pub ttl: u64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct CreateUpdate<T> {
    pub value: T,
    pub name: String,
    pub ttl: u64,
    #[serde(rename = "Type", serialize_with = "ser_recordtype")]
    pub rtype: RecordType,
}

// Bunny has it's own mappings for A, AAAA, TXT, etc.
//
// 0 = A
// 1 = AAAA
// 2 = CNAME
// 3 = TXT
// 4 = MX
// 5 = Redirect
// 6 = Flatten
// 7 = PullZone
// 8 = SRV
// 9 = CAA
// 10 = PTR
// 11 = Script
// 12 = NS
// 13 = SVCB
// 14 = HTTPS
impl TryFrom<u64> for RecordType {
    type Error = Error;

    fn try_from(value: u64) -> std::result::Result<Self, Self::Error> {
        let to = match value {
            0 => RecordType::A,
            1 => RecordType::AAAA,
            2 => RecordType::CNAME,
            3 => RecordType::TXT,
            4 => RecordType::MX,
            // 5 => RecordType::Redirect,
            // 6 => RecordType::Flatten,
            // 7 => RecordType::PullZone,
            8 => RecordType::SRV,
            9 => RecordType::CAA,
            10 => RecordType::PTR,
            // 11 => RecordType::Script,
            12 => RecordType::NS,
            13 => RecordType::SVCB,
            14 => RecordType::HTTPS,
            _ => return Err(Error::ApiError(format!("Invalid RecordType ID {value}")))
        };
        Ok(to)
    }
}

impl From<RecordType> for u64 {
    fn from(value: RecordType) -> Self {
        match value {
            RecordType::A => 0,
            RecordType::AAAA => 1,
            RecordType::CNAME => 2,
            RecordType::TXT => 3,
            RecordType::MX => 4,
            RecordType::SRV => 8,
            RecordType::CAA => 9,
            RecordType::PTR => 10,
            RecordType::NS => 12,
            RecordType::SVCB => 13,
            RecordType::HTTPS => 14,
        }
    }
}

pub(crate) fn de_recordtype<'de, D>(deser: D) -> std::result::Result<RecordType, D::Error>
where
    D: Deserializer<'de>,
{
    let v = u64::deserialize(deser)?;
    RecordType::try_from(v)
       .map_err(serde::de::Error::custom)
}


fn ser_recordtype<S>(rt: &RecordType, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    u64::from(*rt).serialize(serializer)
}
