use hc_lib_ranking_index;
use hdk::prelude::*;

entry_defs![Path::entry_def()];

#[derive(Serialize, Deserialize, Debug)]
pub struct RankEntryInput {
    ranking: i64,
    entry_hash: EntryHash,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRankingInput {
    direction: GetRankingDirection,
    entry_count: usize,
    cursor: Option<GetRankingCursor>,
}

#[hdk_extern]
pub fn rank_entry(input: RankEntryInput) -> ExternResult<()> {
    hc_lib_ranking_index::rank_entry(input.entry_hash, input.ranking)
}

#[hdk_extern]
pub fn get_entry_ranking(input: GetRankingInput) -> ExternResult<GetEntryRankingOutput> {
    hc_lib_ranking_index::get_entry_ranking(input.entry_count, input.direction, input.cursor)
}
