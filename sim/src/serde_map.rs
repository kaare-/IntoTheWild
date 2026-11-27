//! JSON object keys must be strings. Maps keyed by ids are stored as pairs.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

pub fn serialize<K, V, S>(map: &BTreeMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize,
    V: Serialize,
    S: Serializer,
{
    map.iter().collect::<Vec<_>>().serialize(serializer)
}

pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
where
    K: Deserialize<'de> + Ord,
    V: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let pairs = Vec::<(K, V)>::deserialize(deserializer)?;
    Ok(pairs.into_iter().collect())
}
