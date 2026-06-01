use anyhow::Result;
use scheduler::{HeapEntry, Priority, Scheduler, Task};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::BinaryHeap;
use std::sync::Mutex;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let sched = Scheduler::new();
    let heap = Arc::new(Mutex::new(BinaryHeap::new()));
    let seq = Arc::new(AtomicU64::new(0));
    let (submit_tx, submit_rx) = mpsc::channel(128);
    let (cancel_tx, cancel_rx) = mpsc::channel(128);

    let sched_clone = Arc::new(sched);
    let heap_clone = Arc::clone(&heap);
    let seq_clone = Arc::clone(&seq);

    tokio::spawn(async move {
        sched_clone
            .run(heap_clone, seq_clone, submit_rx, cancel_rx)
             .await;
    });

    Ok(());
}
