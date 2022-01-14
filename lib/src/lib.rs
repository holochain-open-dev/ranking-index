use hdk::{hash_path::path::Component, prelude::*};
use std::collections::BTreeMap;

mod types;
pub use types::*;

pub struct RankingIndex {
    pub name: &'static str,
    pub mod_interval: u64,
}

impl RankingIndex {
    pub fn new_with_default_mod(name: &'static str) -> Self {
        RankingIndex {
            name,
            mod_interval: 100,
        }
    }

    pub fn rank_entry(&self, entry_hash: EntryHash, ranking: i64) -> ExternResult<()> {
        let path = self.get_ranking_path(ranking);

        path.ensure()?;

        create_link(
            path.path_entry_hash()?,
            entry_hash,
            ranking_to_tag(ranking)?,
        )?;

        Ok(())
    }

    pub fn get_entry_ranking(
        &self,
        direction: GetRankingDirection,
        entry_count: usize,
        cursor: Option<GetRankingCursor>,
    ) -> ExternResult<EntryRanking> {
        let intervals: BTreeMap<i64, Path> = self.get_interval_paths()?;

        let mut entry_ranking: EntryRanking = BTreeMap::new();
        let mut interval_index = initial_interval_index(&intervals, direction, cursor);

        while ranking_len(&entry_ranking) < entry_count && interval_index < intervals.len() {
            let path_to_fetch =
                intervals.values().into_iter().collect::<Vec<&Path>>()[interval_index];
            let new_entry_ranking = self.get_ranking_from_interval_path(path_to_fetch)?;

            for (ranking, entry_hashes) in new_entry_ranking {
                for entry_hash in entry_hashes {
                    if ranking_len(&entry_ranking) < entry_count {
                        entry_ranking
                            .entry(ranking)
                            .or_insert_with(Vec::new)
                            .push(entry_hash);
                    }
                }
            }
            interval_index += 1;
        }

        Ok(entry_ranking)
    }

    fn get_interval_paths(&self) -> ExternResult<BTreeMap<i64, Path>> {
        let root_path = self.root_path();

        let children_paths = root_path.children_paths()?;

        let mut interval_paths: BTreeMap<i64, Path> = BTreeMap::new();

        for path in children_paths {
            if let Some(component) = path.leaf() {
                if let Ok(ranking) = component_to_ranking(component) {
                    interval_paths.insert(ranking, path);
                }
            }
        }

        Ok(interval_paths)
    }

    fn get_ranking_from_interval_path(&self, interval_path: &Path) -> ExternResult<EntryRanking> {
        let links = get_links(interval_path.path_entry_hash()?, None)?;

        let entry_ranking = links
            .into_iter()
            .map(|link| {
                let ranking = tag_to_ranking(link.tag)?;
                Ok((ranking, link.target))
            })
            .collect::<ExternResult<Vec<(i64, EntryHash)>>>()?;

        let mut ranking_map: EntryRanking = BTreeMap::new();

        for (ranking, entry_hash) in entry_ranking {
            ranking_map
                .entry(ranking)
                .or_insert_with(Vec::new)
                .push(entry_hash);
        }

        Ok(ranking_map)
    }
    /*
       pub fn update_entry_ranking(
           &self,
           entry_hash: EntryHash,
           original_ranking: i64,
           new_ranking: i64,
       ) -> ExternResult<()> {
           unimplemented!()
       }
    */
    fn ranking_mod(&self, ranking: i64) -> i64 {
        ranking % (self.mod_interval as i64)
    }

    fn get_ranking_path(&self, ranking: i64) -> Path {
        Path::from(format!(
            "{}.{}",
            self.root_path_str(),
            self.ranking_mod(ranking)
        ))
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

    Ok(LinkTag(bytes.bytes().clone()))
}

fn tag_to_ranking(tag: LinkTag) -> ExternResult<i64> {
    let bytes = tag.into_inner();
    let sb = SerializedBytes::from(UnsafeBytes::from(bytes));

    let ranking: RankingTag = sb.try_into()?;
    Ok(ranking.ranking)
}

fn component_to_ranking(c: &Component) -> ExternResult<i64> {
    let s: String = c.try_into()?;
    let ranking = s
        .parse::<i64>()
        .or(Err(WasmError::Guest("Bad component".into())))?;

    Ok(ranking)
}

fn ranking_len(entry_ranking: &EntryRanking) -> usize {
    entry_ranking.values().fold(0, |acc, next| acc + next.len())
}

fn initial_interval_index(
    interval_paths: &BTreeMap<i64, Path>,
    direction: GetRankingDirection,
    maybe_cursor: Option<GetRankingCursor>,
) -> usize {
    match maybe_cursor {
        None => match direction {
            GetRankingDirection::Ascendent => 0,
            GetRankingDirection::Descendent => interval_paths.len() - 1,
        },
        Some(cursor) => {
            let ordered_keys: Vec<i64> = interval_paths.keys().into_iter().cloned().collect();
            for i in 0..(interval_paths.len() - 1) {
                if ordered_keys[i] <= cursor.last_seen_ranking
                    && cursor.last_seen_ranking < ordered_keys[i + 1]
                {
                    return i;
                }
            }
            return 0;
        }
    }
}
