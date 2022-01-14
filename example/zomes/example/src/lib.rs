use hc_lib_ranking_index::*;
use hdk::prelude::*;

const MY_INDEX: RankingIndex = RankingIndex {
    name: "my_thing",
    mod_interval: 3,
};

#[hdk_entry(id = "demo")]
pub struct DemoEntry(String);

entry_defs![DemoEntry::entry_def(), Path::entry_def()];

#[derive(Serialize, Deserialize, Debug)]
pub struct RankEntryInput {
    pub ranking: i64,
    pub entry_hash: EntryHash,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRankingInput {
    pub direction: GetRankingDirection,
    pub entry_count: usize,
    pub cursor: Option<GetRankingCursor>,
}

#[hdk_extern]
pub fn rank_entry(input: RankEntryInput) -> ExternResult<()> {
    MY_INDEX.rank_entry(input.entry_hash, input.ranking)
}

#[hdk_extern]
pub fn get_entry_ranking(input: GetRankingInput) -> ExternResult<EntryRanking> {
    MY_INDEX.get_entry_ranking(input.direction, input.entry_count, input.cursor)
}

#[hdk_extern]
pub fn create_demo_entry(entry: DemoEntry) -> ExternResult<EntryHash> {
    create_entry(&entry)?;

    hash_entry(&entry)
}
