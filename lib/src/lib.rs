use hdk::{hash_path::path::Component, prelude::*};
use std::collections::BTreeMap;

mod types;
pub use types::*;

impl RankingIndex {
    pub fn new_with_default_mod(link_type: ScopedLinkType) -> Self {
        RankingIndex {
            link_type,
            index_interval: 100,
        }
    }

    /// Creates a new link between a path of the format
    /// `ranking_by_[RANKING_NAME].[INTERVAL_NUMBER]` and the specified
    /// entry with an optional custom tag.
    ///
    /// INTERVAL_NUMBER is the `ranking` as provided as argument divided
    /// by the `index_interval` of the [`RankingIndex`].
    ///
    /// If the path doesn't exist yet, it will be created on the fly.
    pub fn create_ranking(
        &self,
        hash: AnyLinkableHash,
        ranking: i64,
        tag: Option<SerializedBytes>,
    ) -> ExternResult<()> {
        let path:  Path = self.get_ranking_path(ranking);
        let typed_path = path.typed(self.link_type)?;
        typed_path.ensure()?;

        create_link(
            typed_path.path_entry_hash()?,
            hash,
            self.link_type,
            ranking_to_tag(ranking, tag)?,
        )?;

        Ok(())
    }

    /// Deletes the link associated to an entry ranking for the specified entry.
    pub fn delete_ranking(
        &self,
        hash: AnyLinkableHash,
        ranking: i64,
    ) -> ExternResult<()> {
        // Get previous ranking
        let ranking_path = &self.get_ranking_path(ranking);
        let links = get_links(
            ranking_path.path_entry_hash()?,
            ..,
            None
        )?;

        let links_to_delete: Vec<ActionHash> = links
            .clone()
            .into_iter()
            .filter(|link| link.target.eq(&AnyLinkableHash::from(hash.clone())))
            .map(|link| link.create_link_hash)
            .collect();

        // Delete links for previous ranking
        for to_delete in links_to_delete {
            delete_link(to_delete)?;
        }

        Ok(())
    }

    /// Gets highest/lowest `count` ranked entries. The `direction` specifies
    /// whether to get the highest or the lowest ranked entries.
    ///
    /// The SQL analogue of `get_ranking_chunk(GetRankingDirection::Ascending, 10)`
    /// would be:
    /// `SELECT * FROM all_ranked_entries ORDER BY ranking ASC LIMIT 10`
    ///
    /// Optionally, a `cursor` can be specified in order to get the highest/lowest
    /// `count` ranked entries starting from the ranking specified in the cursor.
    ///
    /// The SQL analogue of
    /// `get_ranking_chunk(GetRankingDirection::Descending, 5, Some( GetRankingCursor { from_ranking: 350 }))`
    /// would be
    /// `WITH ranked_entries_subset AS (SELECT * FROM all_ranked_entries WHERE ranking < 350) SELECT * FROM ranked_entries_subset ORDER BY ranking DESC LIMIT 5`
    pub fn get_ranking_chunk(
        &self,
        direction: GetRankingDirection,
        count: usize,
        cursor: Option<GetRankingCursor>,
    ) -> ExternResult<Ranking> {

        let intervals = self.get_interval_paths()?;

        let mut ranking_map: Ranking = BTreeMap::new();
        let mut interval_index =
            initial_interval_index(&intervals, direction.clone(), cursor.clone()) as isize;

        let paths: Vec<&Path> = intervals.values().into_iter().collect();

        while ranking_len(&ranking_map) < count
            && interval_index >= 0
            && interval_index < intervals.len() as isize
        {
            let path_to_fetch = paths[interval_index as usize];
            let new_ranking = &self.get_ranking_from_interval_path(path_to_fetch)?;

            for (ranking, hashes) in new_ranking {
                if is_inside_query_range(ranking.clone(), direction.clone(), cursor.clone()) {
                    for hash in hashes {
                        ranking_map
                            .entry(ranking.clone())
                            .or_insert_with(Vec::new)
                            .push(hash.clone());
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

        Ok(ranking_map)
    }


    fn get_interval_paths(&self) -> ExternResult<BTreeMap<i64, Path>> {

        let root_path = self.root_path().into_typed(self.link_type);

        let children_paths = root_path.children_paths()?;

        let mut interval_paths: BTreeMap<i64, Path> = BTreeMap::new();

        for path in children_paths {
            if let Some(component) = path.leaf() {
                if let Ok(ranking) = component_to_ranking(component) {
                    interval_paths.insert(ranking, path.path);
                }
            }
        }

        Ok(interval_paths)
    }

    fn get_ranking_from_interval_path(&self, interval_path: &Path) -> ExternResult<Ranking> {

        let links = get_links(
            interval_path.path_entry_hash()?, 
            ..,
            None
        )?;

        let ranking = links
            .into_iter()
            .map(|link| {
                let ranking = tag_to_ranking(link.tag)?;
                Ok((ranking.0, link.target, ranking.1))
            })
            .collect::<ExternResult<Vec<(i64, AnyLinkableHash, Option<SerializedBytes>)>>>()?;

        let mut ranking_map: Ranking = BTreeMap::new();

        for (ranking, hash, custom_tag) in ranking {
            ranking_map
                .entry(ranking)
                .or_insert_with(Vec::new)
                .push(HashWithTag {
                    hash: AnyLinkableHash::from(hash),
                    tag: custom_tag,
                });
        }

        Ok(ranking_map)
    }

    fn ranking_interval(&self, ranking: i64) -> i64 {
        ranking / (self.index_interval as i64)
    }

    fn get_ranking_path(&self, ranking: i64) -> Path {
        let path = Path::from(format!(
            "{}.{}",
            self.root_path_str(),
            self.ranking_interval(ranking)
        ));

        path
    }

    fn root_path(&self) -> Path {
        Path::from(self.root_path_str())
    }

    fn root_path_str(&self) -> String {
        format!("ranking_index")
    }
}

fn ranking_to_tag(ranking: i64, custom_tag: Option<SerializedBytes>) -> ExternResult<LinkTag> {
    let bytes = SerializedBytes::try_from(RankingTag {
        ranking,
        custom_tag,
    }).map_err(|e| wasm_error!(WasmErrorInner::Guest(e.into())))?;

    Ok(LinkTag(bytes.bytes().clone()))
}

fn tag_to_ranking(tag: LinkTag) -> ExternResult<(i64, Option<SerializedBytes>)> {
    let bytes = tag.into_inner();
    let sb = SerializedBytes::from(UnsafeBytes::from(bytes));

    let ranking = types::RankingTag::try_from(sb).map_err(|e| wasm_error!(WasmErrorInner::Guest(e.into())))?;
    Ok((ranking.ranking, ranking.custom_tag))
}

fn component_to_ranking(c: &Component) -> ExternResult<i64> {
    let s = String::try_from(c).map_err(|e| wasm_error!(WasmErrorInner::Guest(e.into())))?;
    let ranking = s
        .parse::<i64>()
        .map_err(|_| wasm_error!(WasmErrorInner::Guest("Bad component".into())))?;

    Ok(ranking)
}

fn ranking_len(ranking: &Ranking) -> usize {
    ranking.values().fold(0, |acc, next| acc + next.len())
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
