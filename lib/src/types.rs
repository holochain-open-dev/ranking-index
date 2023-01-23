use hdk::prelude::*;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GetRankingDirection {
    Ascendent,
    Descendent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRankingCursor {
    pub from_ranking: i64,
}

#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
pub struct RankingTag {
    pub ranking: i64,
    pub custom_tag: Option<SerializedBytes>,
}

#[derive(Serialize, Deserialize, SerializedBytes, Debug, Clone)]
pub struct HashWithTag {
    pub hash: AnyLinkableHash,
    pub tag: Option<SerializedBytes>,
}

pub struct RankingIndex {
    pub link_type: ScopedLinkType,
    pub index_interval: u64,
}

pub type Ranking = BTreeMap<i64, Vec<HashWithTag>>;
