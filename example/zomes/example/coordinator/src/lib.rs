use hc_lib_ranking_index::*;
use example_ranking_index_integrity::*;
use hdk::prelude::*;

pub fn my_ranking_index() -> RankingIndex {
    let my_index: RankingIndex = RankingIndex {
        link_type: ScopedLinkType::try_from(LinkTypes::Ranking).unwrap(),
        index_interval: 3,
    };

    my_index
}

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
    let custom_tag = SerializedBytes::try_from(input.entry_hash.clone())
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.into())))?;

    my_ranking_index().create_entry_ranking(input.entry_hash, input.ranking, Some(custom_tag))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteEntryRankingInput {
    pub current_ranking: i64,
    pub entry_hash: EntryHash,
}

#[hdk_extern]
pub fn delete_entry_ranking(input: DeleteEntryRankingInput) -> ExternResult<()> {
    my_ranking_index().delete_entry_ranking(input.entry_hash, input.current_ranking)
}

#[hdk_extern]
pub fn get_entry_ranking_chunk(input: GetRankingsInput) -> ExternResult<EntryRanking> {
    my_ranking_index().get_entry_ranking_chunk(input.direction, input.entry_count, input.cursor)
}

#[hdk_extern]
pub fn create_demo_entry(content: String) -> ExternResult<EntryHash> {
    let entry = DemoEntry(content);
    create_entry(example_ranking_index_integrity::EntryTypes::DemoEntry(entry.clone()))?;

    hash_entry(&entry)
}
