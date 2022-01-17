use std::collections::BTreeMap;
use hdk::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GetRankingsDirection {
    Ascendent,
    Descendent,
}

pub type EntryRankings = BTreeMap<i64, Vec<EntryHash>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRankingsCursor {
    pub from_ranking: i64,
}