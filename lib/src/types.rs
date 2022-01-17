use std::collections::BTreeMap;
use hdk::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GetRankingsDirection {
    Ascendent,
    Descendent,
}

pub type EntryRanking = BTreeMap<i64, Vec<EntryHash>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRankingsCursor {
    pub from_ranking: i64,
}