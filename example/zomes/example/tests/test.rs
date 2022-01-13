use futures::future::join_all;
use hdk::prelude::{EntryHash, HeaderHash};
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

    for (entry_hash, index) in entry_hashes.enumerate() {
        let input = RankEntryInput {
          ranking: index,
            entry_hash,
        };

        conductors[0].call(&alice_zome, "rank_entry", input).await;
    }

    consistency_10s(&[&alice, &bobbo]).await;

    let get_entry_ranking_input = GetRankingInput {
        direction: GetRankingDirection::Ascendent,
        entry_count: 2,
        cursor: None,
    };
    let ranking_output: GetEntryRankingOutput = conductors[0]
        .call(&alice_zome, "get_entry_ranking", get_entry_ranking_input)
        .await;

    assert_eq!(ranking_output.get(0).unwrap(), entry_hashes[0]);
    assert_eq!(ranking_output.get(1).unwrap(), entry_hashes[1]);
    assert_eq!(ranking_output.len(), 2);

    let get_entry_ranking_input = GetRankingInput {
        direction: GetRankingDirection::Ascendent,
        entry_count: 12,
        cursor: ranking_output.cursor,
    };
    let ranking_output: GetEntryRankingOutput = conductors[0]
        .call(&alice_zome, "get_entry_ranking", get_entry_ranking_input)
        .await;

    assert_eq!(ranking_output.get(0).unwrap(), entry_hashes[3]);
    assert_eq!(ranking_output.get(1).unwrap(), entry_hashes[4]);
    assert_eq!(ranking_output.len(), 6);
}
