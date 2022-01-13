use hdk::prelude::*;

mod types;
pub use types::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct RankingIndex {
    name: String,
    mod_interval: u64,
}

impl RankingIndex {
    pub fn new_with_default_mod(name: String) -> Self {
        RankingIndex {
            name,
            mod_interval: 100,
        }
    }

    pub fn rank_entry(&self, entry_hash: EntryHash, ranking: i64) -> ExternResult<()> {
        let path = self.get_ranking_path(ranking);

        path.ensure()?;

        create_link(path.path_entry_hash()?, entry_hash, ranking_to_tag(ranking)?)?;

        Ok(())
    }

    pub fn get_entry_ranking(
        &self,
        direction: GetRankingDirection,
        entry_count: usize,
        cursor: Option<RankingCursor>,
    ) -> ExternResult<GetEntryRankingOutput> {
        let root_path = self.root_path();

        let children_paths = root_path.children_paths()?;
        
        
    }


    fn ranking_mod(&self, ranking: i64) -> i64 {
        ranking % (self.mod_interval as i64)
    }

    fn get_ranking_path(&self, ranking: i64) -> Path {
        Path::from(format!("{}.{}", self.root_path_str(), self.ranking_mod(ranking)))
    }

    fn root_path(&self) -> Path {
        Path::from(self.root_path_str())
    }

    fn root_path_str(&self) -> String {
        format!("ranking_by_{}", self.name)
    }
}

#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
struct RankingTag {
    ranking: i64,
}

fn ranking_to_tag(ranking: i64) -> ExternResult<LinkTag> {
    let bytes = SerializedBytes::try_from(RankingTag { ranking })?;

    Ok(LinkTag(bytes.bytes()))
}

fn tag_to_ranking(tag: LinkTag) -> ExternResult<i64> {
    let bytes = tag.into_inner();
    let sb = SerializedBytes::from(UnsafeBytes::from(bytes));

    let ranking: RankingTag = sb.try_into()?;
    Ok(ranking.ranking)
}
