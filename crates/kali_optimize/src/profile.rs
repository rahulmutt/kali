use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Current profile-data format version.
pub const PROFILE_DATA_VERSION: u32 = 1;

/// Stable profile-sample kinds used by the later PGO pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileSampleKind {
    Function,
    Branch,
    Layout,
}

/// One deterministic profile sample.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSample {
    pub kind: ProfileSampleKind,
    pub key: String,
    pub weight: u64,
}

impl ProfileSample {
    /// Create a normalized profile sample.
    pub fn new(kind: ProfileSampleKind, key: impl Into<String>, weight: u64) -> Self {
        Self {
            kind,
            key: key.into().trim().to_string(),
            weight,
        }
    }
}

/// Stable PGO profile data with deterministic normalization rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileData {
    pub version: u32,
    #[serde(default)]
    pub samples: Vec<ProfileSample>,
}

impl Default for ProfileData {
    fn default() -> Self {
        Self {
            version: PROFILE_DATA_VERSION,
            samples: Vec::new(),
        }
    }
}

impl ProfileData {
    /// Create a normalized profile snapshot using the current format version.
    pub fn new(samples: impl Into<Vec<ProfileSample>>) -> Self {
        Self {
            version: PROFILE_DATA_VERSION,
            samples: samples.into(),
        }
        .normalized()
    }

    /// Return a canonicalized copy of this profile snapshot.
    pub fn normalized(mut self) -> Self {
        self.version = PROFILE_DATA_VERSION;
        self.samples = normalize_samples(self.samples);
        self
    }

    /// Merge two snapshots with deterministic aggregation.
    pub fn merge(&self, other: &Self) -> Self {
        let mut samples = self.samples.clone();
        samples.extend(other.samples.iter().cloned());
        Self::new(samples)
    }

    /// Append a sample and re-canonicalize the snapshot.
    pub fn push(&mut self, sample: ProfileSample) {
        self.samples.push(sample);
        *self = self.clone().normalized();
    }

    /// Whether the snapshot already uses the current format version.
    pub fn is_current_version(&self) -> bool {
        self.version == PROFILE_DATA_VERSION
    }

    /// Return all function hot paths that meet the minimum recorded weight.
    pub fn hot_function_keys(&self, minimum_weight: u64) -> Vec<String> {
        self.hot_keys(ProfileSampleKind::Function, minimum_weight)
    }

    /// Return all hot paths for the requested sample kind that meet the minimum recorded weight.
    pub fn hot_keys(&self, kind: ProfileSampleKind, minimum_weight: u64) -> Vec<String> {
        self.samples
            .iter()
            .filter(|sample| sample.kind == kind && sample.weight >= minimum_weight)
            .map(|sample| sample.key.clone())
            .collect()
    }

    /// Return the accumulated weight for a deterministic sample key.
    pub fn sample_weight(&self, kind: ProfileSampleKind, key: &str) -> Option<u64> {
        self.samples
            .iter()
            .find(|sample| sample.kind == kind && sample.key == key)
            .map(|sample| sample.weight)
    }
}

fn normalize_samples(samples: Vec<ProfileSample>) -> Vec<ProfileSample> {
    let mut merged: BTreeMap<(ProfileSampleKind, String), u64> = BTreeMap::new();
    for sample in samples {
        let key = sample.key.trim();
        if key.is_empty() || sample.weight == 0 {
            continue;
        }
        merged
            .entry((sample.kind, key.to_string()))
            .and_modify(|weight| *weight = weight.saturating_add(sample.weight))
            .or_insert(sample.weight);
    }

    merged
        .into_iter()
        .map(|((kind, key), weight)| ProfileSample { kind, key, weight })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_data_normalizes_samples_deterministically() {
        let profile = ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Layout, "layout:point", 1),
            ProfileSample::new(ProfileSampleKind::Function, " hot-path ", 2),
            ProfileSample::new(ProfileSampleKind::Function, "hot-path", 5),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:if-true", 3),
            ProfileSample::new(ProfileSampleKind::Function, "", 42),
            ProfileSample::new(ProfileSampleKind::Function, "cold-path", 0),
        ]);

        assert!(profile.is_current_version());
        assert_eq!(
            profile.samples,
            vec![
                ProfileSample::new(ProfileSampleKind::Function, "hot-path", 7),
                ProfileSample::new(ProfileSampleKind::Branch, "branch:if-true", 3),
                ProfileSample::new(ProfileSampleKind::Layout, "layout:point", 1),
            ]
        );
        assert_eq!(
            profile.sample_weight(ProfileSampleKind::Function, "hot-path"),
            Some(7)
        );
        assert_eq!(
            profile.sample_weight(ProfileSampleKind::Function, "cold-path"),
            None
        );
    }

    #[test]
    fn profile_data_merge_is_order_invariant() {
        let left = ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Function, "alpha", 2),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:0", 4),
        ]);
        let right = ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Function, "alpha", 3),
            ProfileSample::new(ProfileSampleKind::Function, "beta", 1),
        ]);

        let merged = left.merge(&right);
        let reversed = right.merge(&left);

        assert_eq!(merged, reversed);
        assert_eq!(
            merged.samples,
            vec![
                ProfileSample::new(ProfileSampleKind::Function, "alpha", 5),
                ProfileSample::new(ProfileSampleKind::Function, "beta", 1),
                ProfileSample::new(ProfileSampleKind::Branch, "branch:0", 4),
            ]
        );
    }

    #[test]
    fn profile_data_hot_function_keys_filter_by_threshold() {
        let profile = ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Function, "hot", 9),
            ProfileSample::new(ProfileSampleKind::Function, "warm", 3),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:0", 8),
        ]);

        assert_eq!(profile.hot_function_keys(4), vec!["hot".to_string()]);
        assert_eq!(
            profile.hot_function_keys(1),
            vec!["hot".to_string(), "warm".to_string()]
        );
    }

    #[test]
    fn profile_data_hot_keys_filter_by_kind() {
        let profile = ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Function, "hot", 9),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:0", 8),
            ProfileSample::new(ProfileSampleKind::Layout, "layout:point", 7),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:1", 3),
        ]);

        assert_eq!(
            profile.hot_keys(ProfileSampleKind::Branch, 4),
            vec!["branch:0".to_string()]
        );
        assert_eq!(
            profile.hot_keys(ProfileSampleKind::Layout, 1),
            vec!["layout:point".to_string()]
        );
    }

    #[test]
    fn profile_data_round_trips_through_json() {
        let profile = ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Layout, "layout:point", 7),
            ProfileSample::new(ProfileSampleKind::Function, "hot", 4),
        ]);

        let json = serde_json::to_string(&profile).expect("serialize profile data");
        assert_eq!(
            json,
            r#"{"version":1,"samples":[{"kind":"function","key":"hot","weight":4},{"kind":"layout","key":"layout:point","weight":7}]}"#
        );
        let parsed: ProfileData = serde_json::from_str(&json).expect("deserialize profile data");
        assert_eq!(parsed, profile);
    }
}
