use super::*;

fn cluster(name: &str, tier: u8, reachable: Option<u64>) -> Cluster {
    Cluster {
        name: name.into(),
        entries: vec![],
        tier,
        reachable,
    }
}

#[test]
fn a_worse_tier_at_equal_frequency_dominates() {
    let worse = cluster("a", 1, Some(10));
    let better = cluster("b", 2, Some(10));
    assert!(dominates(&worse, &better));
    assert!(!dominates(&better, &worse));
}

#[test]
fn a_higher_frequency_at_equal_tier_dominates() {
    assert!(dominates(
        &cluster("a", 2, Some(50)),
        &cluster("b", 2, Some(10))
    ));
}

#[test]
fn neither_dominates_when_each_wins_one_axis() {
    // This is the case a total order would have to break with an invented
    // weight. Both land in the same band instead.
    let a = cluster("a", 1, Some(2));
    let b = cluster("b", 2, Some(90));
    assert!(!dominates(&a, &b));
    assert!(!dominates(&b, &a));
}

#[test]
fn an_identical_pair_does_not_dominate_either_way() {
    let a = cluster("a", 2, Some(10));
    let b = cluster("b", 2, Some(10));
    assert!(!dominates(&a, &b));
    assert!(!dominates(&b, &a));
}

#[test]
fn uncountable_clusters_never_participate_in_dominance() {
    let counted = cluster("a", 1, Some(100));
    let unknown = cluster("b", 1, None);
    assert!(!dominates(&counted, &unknown));
    assert!(!dominates(&unknown, &counted));
}

#[test]
fn banding_peels_successive_pareto_frontiers() {
    let clusters = vec![
        cluster("top", 1, Some(50)),
        cluster("wide", 2, Some(90)),
        cluster("mid", 2, Some(50)),
        cluster("low", 3, Some(1)),
    ];
    let bands = band(&clusters);
    assert_eq!(bands.len(), 3, "expected three frontiers, got {bands:?}");
    assert_eq!(
        bands[0].iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        ["top", "wide"]
    );
    assert_eq!(
        bands[1].iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        ["mid"]
    );
    assert_eq!(
        bands[2].iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        ["low"]
    );
}

#[test]
fn banding_an_empty_input_yields_no_bands() {
    assert!(band(&[]).is_empty());
}

#[test]
fn banding_all_uncountable_yields_one_band() {
    // With no frequency axis, nothing dominates anything, so every cluster is
    // on the first frontier. That is the honest outcome, not a ranking.
    let clusters = vec![cluster("a", 1, None), cluster("b", 3, None)];
    let bands = band(&clusters);
    assert_eq!(bands.len(), 1);
    assert_eq!(bands[0].len(), 2);
}

#[test]
fn aggregation_takes_the_worst_tier_and_sums_reachable() {
    let entries = vec![
        ScoredEntry {
            id: "R-02".into(),
            tier: 1,
            reachable: Some(4),
        },
        ScoredEntry {
            id: "R-03".into(),
            tier: 2,
            reachable: Some(6),
        },
    ];
    let clusters = vec![(
        "call-lowering-choke".to_string(),
        vec!["R-02".to_string(), "R-03".to_string()],
    )];
    let aggregated = aggregate(&entries, &clusters);
    assert_eq!(aggregated.len(), 1);
    assert_eq!(aggregated[0].tier, 1, "the worst tier in the cluster wins");
    assert_eq!(aggregated[0].reachable, Some(10));
}

#[test]
fn a_cluster_with_any_uncountable_member_is_uncountable() {
    // Summing over a partially-counted cluster would publish a number smaller
    // than the truth while looking complete.
    let entries = vec![
        ScoredEntry {
            id: "R-15".into(),
            tier: 2,
            reachable: Some(4),
        },
        ScoredEntry {
            id: "R-16".into(),
            tier: 2,
            reachable: None,
        },
    ];
    let clusters = vec![(
        "string-repr".to_string(),
        vec!["R-15".to_string(), "R-16".to_string()],
    )];
    assert_eq!(aggregate(&entries, &clusters)[0].reachable, None);
}

#[test]
fn aggregation_rejects_a_cluster_naming_an_unscored_entry() {
    let entries = vec![ScoredEntry {
        id: "R-02".into(),
        tier: 1,
        reachable: Some(4),
    }];
    let clusters = vec![(
        "c".to_string(),
        vec!["R-02".to_string(), "R-99".to_string()],
    )];
    let result = std::panic::catch_unwind(|| aggregate(&entries, &clusters));
    assert!(
        result.is_err(),
        "an unscored member must not be silently dropped"
    );
}
