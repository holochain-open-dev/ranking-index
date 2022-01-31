use hc_lib_ranking_index::*;
use hdk::prelude::*;

const MY_INDEX: RankingIndex = RankingIndex {
    name: "my_thing",
    index_interval: 3,
};

#[hdk_entry(id = "demo")]
pub struct DemoEntry(String);

entry_defs![DemoEntry::entry_def(), PathEntry::entry_def()];

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateEntryRankingInput {
    pub ranking: i64,
    pub entry_hash: EntryHash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRankingsInput {
    pub direction: GetRankingDirection,
    pub entry_count: usize,
    pub cursor: Option<GetRankingCursor>,
}

#[hdk_extern]
pub fn create_entry_ranking(input: CreateEntryRankingInput) -> ExternResult<()> {
    let custom_tag = SerializedBytes::try_from(input.entry_hash.clone())?;

    MY_INDEX.create_entry_ranking(input.entry_hash, input.ranking, Some(custom_tag))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteEntryRankingInput {
    pub current_ranking: i64,
    pub entry_hash: EntryHash,
}

#[hdk_extern]
pub fn delete_entry_ranking(input: DeleteEntryRankingInput) -> ExternResult<()> {
    MY_INDEX.delete_entry_ranking(input.entry_hash, input.current_ranking)
}

#[hdk_extern]
pub fn get_entry_ranking_chunk(input: GetRankingsInput) -> ExternResult<EntryRanking> {
    MY_INDEX.get_entry_ranking_chunk(input.direction, input.entry_count, input.cursor)
}

#[hdk_extern]
pub fn create_demo_entry(entry: DemoEntry) -> ExternResult<EntryHash> {
    create_entry(&entry)?;

    hash_entry(&entry)
}
