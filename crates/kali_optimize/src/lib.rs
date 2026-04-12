//! Optimization passes for the Kali compiler.

use kali_lir::LirNodeId;

/// Optimization level.
#[derive(Default, Clone, Debug)]
pub enum OptimizationLevel {
    Fast,
    Release,
    ReleaseAdvanced,

    #[default]
    Default,
}

/// Optimizer context.
pub struct Optimizer {
    level: OptimizationLevel,
}

impl Optimizer {
    pub fn new(level: OptimizationLevel) -> Self {
        Self { level }
    }

    pub fn optimize(&self, _lir: LirNodeId) -> LirNodeId {
        match self.level {
            OptimizationLevel::Fast
            | OptimizationLevel::Release
            | OptimizationLevel::ReleaseAdvanced
            | OptimizationLevel::Default => LirNodeId::new(0),
        }
    }
}
