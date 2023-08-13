use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileList {
    pub files: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Proof {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub id: u32,
    pub name: String,
    #[serde(with = "base64")]
    pub content: Vec<u8>,
    pub proof: Proof,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewFile {
    #[serde(with = "base64")]
    pub content: Vec<u8>,
    pub name: String,
}

mod base64 {
    use base64::Engine;
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = base64::engine::general_purpose::STANDARD.encode(v);
        String::serialize(&base64, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        base64::engine::general_purpose::STANDARD
            .decode(base64.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}
