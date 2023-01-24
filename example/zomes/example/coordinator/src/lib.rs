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
    pub hash: EntryHash,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateActionRankingInput {
    pub ranking: i64,
    pub hash: ActionHash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRankingsInput {
    pub direction: GetRankingDirection,
    pub count: usize,
    pub cursor: Option<GetRankingCursor>,
}

#[hdk_extern]
pub fn create_ranking(input: CreateEntryRankingInput) -> ExternResult<()> {
    let custom_tag = SerializedBytes::try_from(input.hash.clone())
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.into())))?;

    my_ranking_index().create_ranking(AnyLinkableHash::from(input.hash), input.ranking, Some(custom_tag))
}

#[hdk_extern]
pub fn create_action_ranking(input: CreateActionRankingInput) -> ExternResult<()> {
    let custom_tag = SerializedBytes::try_from(input.hash.clone())
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.into())))?;

    my_ranking_index().create_ranking(AnyLinkableHash::from(input.hash), input.ranking, Some(custom_tag))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteEntryRankingInput {
    pub current_ranking: i64,
    pub hash: EntryHash,
}

#[hdk_extern]
pub fn delete_ranking(input: DeleteEntryRankingInput) -> ExternResult<()> {
    my_ranking_index().delete_ranking(AnyLinkableHash::from(input.hash), input.current_ranking)
}

#[hdk_extern]
pub fn get_ranking_chunk(input: GetRankingsInput) -> ExternResult<Ranking> {
    my_ranking_index().get_ranking_chunk(input.direction, input.count, input.cursor)
}

#[hdk_extern]
pub fn create_demo_entry(content: String) -> ExternResult<EntryHash> {
    let entry = DemoEntry(content);
    create_entry(example_ranking_index_integrity::EntryTypes::DemoEntry(entry.clone()))?;

    hash_entry(&entry)
}

#[hdk_extern]
pub fn create_demo_entry_get_ah(content: String) -> ExternResult<ActionHash> {
    let entry = DemoEntry(content);
    let action_hash = create_entry(example_ranking_index_integrity::EntryTypes::DemoEntry(entry.clone()))?;

    Ok(action_hash)
}
