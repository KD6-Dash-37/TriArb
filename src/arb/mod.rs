// src/arb/mod.rs
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc::Receiver;

use crate::parse::TopOfBookUpdate;

pub mod naive;
pub use naive::Triangle;

pub trait ArbEvaluator: Send + Sync {
    fn process_update(&self, update: &TopOfBookUpdate);
}

pub async fn arb_loop(
    mut rx: Receiver<TopOfBookUpdate>,
    evaluator: Arc<dyn ArbEvaluator>,
) -> Result<()> {
    while let Some(update) = rx.recv().await {
        evaluator.process_update(&update);
    }
    Ok(())
}
