use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha3::digest::{FixedOutput, Output};
use sha3::{Digest, Sha3_256};
use std::fmt::{Display, Formatter};
use std::str;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Hash(Output<Sha3_256>);

pub fn hash_content(content: impl AsRef<[u8]>) -> Hash {
    Hash(Sha3_256::digest(content))
}

pub fn hash_both(hash1: &Hash, hash2: &Hash) -> Hash {
    let mut hasher = Sha3_256::new();
    hasher.update(hash1.0.as_slice());
    hasher.update(hash2.0.as_slice());
    Hash(hasher.finalize_fixed())
}

impl FromStr for Hash {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;

        let output = Output::<Sha3_256>::clone_from_slice(&bytes);
        Ok(Hash(output))
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = hex::encode(self.0);
        f.serialize_str(&str)
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        Hash::from_str(&str).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_display() {
        assert_eq!(
            hash_content(b"123").to_string(),
            "a03ab19b866fc585b5cb1812a2f63ca861e7e7643ee5d43fd7106b623725fd67".to_string()
        )
    }

    #[test]
    fn test_parse() {
        let parsed_hash =
            Hash::from_str("a03ab19b866fc585b5cb1812a2f63ca861e7e7643ee5d43fd7106b623725fd67")
                .expect("should parse");
        assert_eq!(
            parsed_hash.to_string(),
            "a03ab19b866fc585b5cb1812a2f63ca861e7e7643ee5d43fd7106b623725fd67".to_string()
        )
    }
}
