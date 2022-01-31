use hdk::prelude::*;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GetRankingDirection {
    Ascendent,
    Descendent,
}

pub type EntryRanking = BTreeMap<i64, Vec<EntryHashWithTag>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntryHashWithTag {
    pub entry_hash: EntryHash,
    pub tag: Option<SerializedBytes>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRankingCursor {
    pub from_ranking: i64,
}
