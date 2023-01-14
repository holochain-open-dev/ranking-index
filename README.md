# ranking-index

## Installing

Add this in the `Cargo.toml` of your zome:

```toml
[dependencies]
hc_lib_ranking_index = {git = "https://github.com/holochain-open-dev/ranking-index", branch = "main", package = "hc_lib_ranking_index"}
```

## Usage

1. Define your index:

```rust
const MY_RANKING_INDEX: RankingIndex = RankingIndex {
    name: "my_thing",
    index_interval: 3,
};
```

Here, the `name` identifies the index, so only entries ranked by this index will be returned with `get_entry_ranking_chunk`.

2. Add a LinkType to your zome that will be used as all segments of the index Path

```rust
#[hdk_link_types]
pub enum LinkTypes {
    Ranking
}
```

3. Add an entry to the index with `create_entry_ranking`:

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct RankEntryInput {
    pub ranking: i64,
    pub entry_hash: EntryHash,
}

#[hdk_extern]
pub fn create_entry_ranking(input: RankEntryInput) -> ExternResult<()> {
    MY_RANKING_INDEX.create_entry_ranking(input.entry_hash, input.ranking, None, LinkTypes::Ranking)
}
```

4. Get the ranking of entries with `get_entry_ranking_chunk`:

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct GetRankingInput {
    pub direction: GetRankingDirection,
    pub entry_count: usize,
    pub cursor: Option<GetRankingCursor>,
}


#[hdk_extern]
pub fn get_entry_ranking_chunk(input: GetRankingInput) -> ExternResult<EntryRanking> {
    MY_RANKING_INDEX.get_entry_ranking_chunk(input.direction, input.entry_count, input.cursor)
}
```

---

You can see a fully working example zome [here](/example/zomes/example).

# Dev Setup

Enter the nix-shell by running `nix-shell` in the root folder of the repository..

## Testing

Run the tests with:

```bash
sh run-tests.sh
```