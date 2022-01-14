use example_ranking_index::{GetRankingInput, RankEntryInput};
use hc_lib_ranking_index::{EntryRanking, GetRankingCursor, GetRankingDirection};
use hdk::prelude::EntryHash;
use holochain::test_utils::consistency_10s;
use holochain::{conductor::config::ConductorConfig, sweettest::*};

#[tokio::test(flavor = "multi_thread")]
async fn basic_rank() {
    // Use prebuilt DNA file
    let dna_path = std::env::current_dir()
        .unwrap()
        .join("../../workdir/example-ranking-index.dna");
    let dna = SweetDnaFile::from_bundle(&dna_path).await.unwrap();

    // Set up conductors
    let mut conductors = SweetConductorBatch::from_config(2, ConductorConfig::default()).await;
    let apps = conductors.setup_app("example", &[dna]).await.unwrap();
    conductors.exchange_peer_info().await;

    let ((alice,), (bobbo,)) = apps.into_tuples();

    let alice_zome = alice.zome("example_ranking_index");
    let bob_zome = bobbo.zome("example_ranking_index");

    let entry_contents = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
    let mut entry_hashes: Vec<EntryHash> = vec![];

    for content in entry_contents {
        let hash = conductors[0]
            .call(&alice_zome, "create_entry", String::from(content).clone())
            .await;

        entry_hashes.push(hash);
    }

    for (index, entry_hash) in entry_hashes.clone().into_iter().enumerate() {
        let input = RankEntryInput {
            ranking: index as i64,
            entry_hash,
        };

        let _r: () = conductors[0].call(&alice_zome, "rank_entry", input).await;
    }

    consistency_10s(&[&alice, &bobbo]).await;

    let get_entry_ranking_input = GetRankingInput {
        direction: GetRankingDirection::Ascendent,
        entry_count: 2,
        cursor: None,
    };
    let ranking_output: EntryRanking = conductors[0]
        .call(&alice_zome, "get_entry_ranking", get_entry_ranking_input)
        .await;

    assert_eq!(
        ranking_output.get(&0).unwrap().clone(),
        vec![entry_hashes[0].clone()]
    );
    assert_eq!(
        ranking_output.get(&1).unwrap().clone(),
        vec![entry_hashes[1].clone()]
    );
    assert_eq!(ranking_output.len(), 2);

    let get_entry_ranking_input = GetRankingInput {
        direction: GetRankingDirection::Ascendent,
        entry_count: 12,
        cursor: Some(GetRankingCursor {
            last_seen_ranking: 1,
            last_seen_entry_hash: entry_hashes[1].clone(),
        }),
    };
    let ranking_output: EntryRanking = conductors[1]
        .call(&bob_zome, "get_entry_ranking", get_entry_ranking_input)
        .await;

    assert_eq!(
        ranking_output.get(&2).unwrap().clone(),
        vec![entry_hashes[3].clone()]
    );
    assert_eq!(
        ranking_output.get(&3).unwrap().clone(),
        vec![entry_hashes[4].clone()]
    );
    assert_eq!(ranking_output.len(), 6);


    let get_entry_ranking_input = GetRankingInput {
        direction: GetRankingDirection::Descendent,
        entry_count: 2,
        cursor: None,
    };
    let ranking_output: EntryRanking = conductors[0]
        .call(&alice_zome, "get_entry_ranking", get_entry_ranking_input)
        .await;

    assert_eq!(
        ranking_output.get(&7).unwrap().clone(),
        vec![entry_hashes[7].clone()]
    );
    assert_eq!(
        ranking_output.get(&6).unwrap().clone(),
        vec![entry_hashes[6].clone()]
    );
    assert_eq!(ranking_output.len(), 2);
}
