use hdk::{hash_path::path::Component, prelude::*};
use std::collections::BTreeMap;

mod types;
pub use types::*;

pub struct RankingIndex {
    pub name: &'static str,
    pub index_interval: u64,
}

impl RankingIndex {
    pub fn new_with_default_mod(name: &'static str) -> Self {
        RankingIndex {
            name,
            index_interval: 100,
        }
    }

    pub fn create_entry_ranking(
        &self,
        entry_hash: EntryHash,
        ranking: i64,
        tag: Option<SerializedBytes>,
    ) -> ExternResult<()> {
        let path = self.get_ranking_path(ranking);

        path.ensure()?;

        create_link(
            path.path_entry_hash()?,
            entry_hash,
            ranking_to_tag(ranking, tag)?,
        )?;

        Ok(())
    }

    pub fn delete_entry_ranking(
        self,
        entry_hash: EntryHash,
        entry_ranking: i64,
    ) -> ExternResult<()> {
        // Get previous ranking
        let ranking_path = self.get_ranking_path(entry_ranking);
        let links = get_links(ranking_path.path_entry_hash()?, None)?;

        let links_to_delete: Vec<HeaderHash> = links
            .clone()
            .into_iter()
            .filter(|link| link.target.eq(&entry_hash))
            .map(|link| link.create_link_hash)
            .collect();

        // Delete links for previous ranking
        for to_delete in links_to_delete {
            delete_link(to_delete)?;
        }

        Ok(())
    }

    pub fn get_entry_ranking_chunk(
        &self,
        direction: GetRankingDirection,
        entry_count: usize,
        cursor: Option<GetRankingCursor>,
    ) -> ExternResult<EntryRanking> {
        let intervals: BTreeMap<i64, Path> = self.get_interval_paths()?;

        let mut entry_ranking: EntryRanking = BTreeMap::new();
        let mut interval_index =
            initial_interval_index(&intervals, direction.clone(), cursor.clone()) as isize;

        let paths: Vec<&Path> = intervals.values().into_iter().collect();

        while ranking_len(&entry_ranking) < entry_count
            && interval_index >= 0
            && interval_index < intervals.len() as isize
        {
            let path_to_fetch = paths[interval_index as usize];
            let new_entry_ranking = self.get_ranking_from_interval_path(path_to_fetch)?;

            for (ranking, entry_hashes) in new_entry_ranking {
                if is_inside_query_range(ranking, direction.clone(), cursor.clone()) {
                    for entry_hash in entry_hashes {
                        entry_ranking
                            .entry(ranking)
                            .or_insert_with(Vec::new)
                            .push(entry_hash);
                    }
                }
            }

            match direction {
                GetRankingDirection::Ascendent => {
                    interval_index += 1;
                }
                GetRankingDirection::Descendent => {
                    interval_index -= 1;
                }
            }
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
                Ok((ranking.0, link.target, ranking.1))
            })
            .collect::<ExternResult<Vec<(i64, EntryHash, Option<SerializedBytes>)>>>()?;

        let mut ranking_map: EntryRanking = BTreeMap::new();

        for (ranking, entry_hash, custom_tag) in entry_ranking {
            ranking_map
                .entry(ranking)
                .or_insert_with(Vec::new)
                .push(EntryHashWithTag {
                    entry_hash,
                    tag: custom_tag,
                });
        }

        Ok(ranking_map)
    }

    fn ranking_interval(&self, ranking: i64) -> i64 {
        ranking / (self.index_interval as i64)
    }

    fn get_ranking_path(&self, ranking: i64) -> Path {
        Path::from(format!(
            "{}.{}",
            self.root_path_str(),
            self.ranking_interval(ranking)
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
    custom_tag: Option<SerializedBytes>,
}

fn ranking_to_tag(ranking: i64, custom_tag: Option<SerializedBytes>) -> ExternResult<LinkTag> {
    let bytes = SerializedBytes::try_from(RankingTag {
        ranking,
        custom_tag,
    })?;

    Ok(LinkTag(bytes.bytes().clone()))
}

fn tag_to_ranking(tag: LinkTag) -> ExternResult<(i64, Option<SerializedBytes>)> {
    let bytes = tag.into_inner();
    let sb = SerializedBytes::from(UnsafeBytes::from(bytes));

    let ranking: RankingTag = sb.try_into()?;
    Ok((ranking.ranking, ranking.custom_tag))
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
                if ordered_keys[i] <= cursor.from_ranking
                    && cursor.from_ranking < ordered_keys[i + 1]
                {
                    return i;
                }
            }
            return 0;
        }
    }
}

fn is_inside_query_range(
    ranking: i64,
    direction: GetRankingDirection,
    maybe_cursor: Option<GetRankingCursor>,
) -> bool {
    match maybe_cursor {
        None => true,
        Some(cursor) => {
            let from_ranking = cursor.from_ranking;
            match direction {
                GetRankingDirection::Ascendent => ranking >= from_ranking,
                GetRankingDirection::Descendent => ranking <= from_ranking,
            }
        }
    }
}
