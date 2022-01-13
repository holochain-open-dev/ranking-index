use hdk::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum GetRankingDirection {
    Ascendent,
    Descendent,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RankingCursor {
    last_ranking_seen: i64,
    last_entry_seen: EntryHash,
}

pub type EntryRanking = BTreeMap<i64, EntryHash>;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetEntryRankingOutput {
    entry_ranking: EntryRanking,
    cursor: RankingCursor,
}
