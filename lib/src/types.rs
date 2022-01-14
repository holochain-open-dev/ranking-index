use std::collections::BTreeMap;
use hdk::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum GetRankingDirection {
    Ascendent,
    Descendent,
}

pub type EntryRanking = BTreeMap<i64, Vec<EntryHash>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRankingCursor {
    pub last_seen_ranking: i64,
    pub last_seen_entry_hash: EntryHash
}