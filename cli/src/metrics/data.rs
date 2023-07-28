use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use workflow_core::{enums::Describe, sendable::Sendable};

#[derive(Describe, Debug, Clone, Eq, PartialEq, Hash, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum Metric {
    BlocksSubmitted,
    HeaderCount,
    DepCounts,
    BodyCounts,
    TxnCounts,
    ChainBlockCounts,
    MassCounts,
    BlockCount,
    TipHashes,
    Difficulty,
    PastMedianTime,
    VirtualParentHashes,
    VirtualDaaScore,
}

#[derive(Default, Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct MetricsData {
    pub unixtime: f64,
    // ---
    pub blocks_submitted: u64,
    pub header_counts: u64,
    pub dep_counts: u64,
    pub body_counts: u64,
    pub txs_counts: u64,
    pub chain_block_counts: u64,
    pub mass_counts: u64,
    // ---
    pub block_count: u64,
    pub tip_hashes: usize,
    pub difficulty: f64,
    pub past_median_time: u64,
    pub virtual_parent_hashes: usize,
    pub virtual_daa_score: u64,
}

impl MetricsData {
    pub fn new(unixtime: f64) -> Self {
        Self { unixtime, ..Default::default() }
    }
}

#[derive(Default, Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub unixtime: f64,
    pub duration: f64,
    // ---
    pub blocks_submitted: f64,
    pub header_counts: f64,
    pub dep_counts: f64,
    pub body_counts: f64,
    pub txs_counts: f64,
    pub chain_block_counts: f64,
    pub mass_counts: f64,
    // ---
    pub block_count: f64,
    pub tip_hashes: f64,
    pub difficulty: f64,
    pub past_median_time: f64,
    pub virtual_parent_hashes: f64,
    pub virtual_daa_score: f64,
}

impl MetricsSnapshot {
    pub fn get(&self, metric: &Metric) -> Sendable<JsValue> {
        let v = match metric {
            Metric::BlocksSubmitted => JsValue::from(self.blocks_submitted as f64),
            Metric::HeaderCount => JsValue::from(self.header_counts as f64),
            Metric::DepCounts => JsValue::from(self.dep_counts as f64),
            Metric::BodyCounts => JsValue::from(self.body_counts as f64),
            Metric::TxnCounts => JsValue::from(self.txs_counts as f64),
            Metric::ChainBlockCounts => JsValue::from(self.chain_block_counts as f64),
            Metric::MassCounts => JsValue::from(self.mass_counts as f64),
            Metric::BlockCount => JsValue::from(self.block_count as f64),
            Metric::TipHashes => JsValue::from(self.tip_hashes as f64),
            Metric::Difficulty => JsValue::from(self.difficulty),
            Metric::PastMedianTime => JsValue::from(self.past_median_time as f64),
            Metric::VirtualParentHashes => JsValue::from(self.virtual_parent_hashes as f64),
            Metric::VirtualDaaScore => JsValue::from(self.virtual_daa_score as f64),
        };

        Sendable(v)
    }
}

impl From<(&MetricsData, &MetricsData)> for MetricsSnapshot {
    fn from((a, b): (&MetricsData, &MetricsData)) -> Self {
        Self {
            unixtime: b.unixtime,
            duration: b.unixtime - a.unixtime,
            // ---
            blocks_submitted: b.blocks_submitted as f64,     // - a.blocks_submitted,
            header_counts: b.header_counts as f64,           // - a.header_counts,
            dep_counts: b.dep_counts as f64,                 // - a.dep_counts,
            body_counts: b.body_counts as f64,               // - a.body_counts,
            txs_counts: b.txs_counts as f64,                 // - a.txs_counts,
            chain_block_counts: b.chain_block_counts as f64, // - a.chain_block_counts,
            mass_counts: b.mass_counts as f64,               // - a.mass_counts,
            // ---
            block_count: b.block_count as f64,                     // - a.block_count,
            tip_hashes: b.tip_hashes as f64,                       // - a.tip_hashes,
            difficulty: b.difficulty as f64,                       // - a.difficulty,
            past_median_time: b.past_median_time as f64,           // - a.past_median_time,
            virtual_parent_hashes: b.virtual_parent_hashes as f64, // - a.virtual_parent_hashes,
            virtual_daa_score: b.virtual_daa_score as f64,         // - a.virtual_daa_score,
        }
    }
}
