use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
mod scheduler;
use scheduler::{HeapEntry, Priority, Scheduler, Task};

#[tokio::main]
async fn main() {
    let sched = Arc::new(Scheduler::new());
    let heap = Arc::new(Mutex::new(BinaryHeap::new()));
    let seq = Arc::new(Mutex::new(0));

    let (submit_tx, submit_rx) = mpsc::channel::<Task>(100);
    let (cancel_tx, cancel_rx) = mpsc::channel::<u64>(100);

    // Clona antes de mover pro spawn
    let sched_clone = Arc::clone(&sched);
    let heap_clone = Arc::clone(&heap);
    let seq_clone = Arc::clone(&seq);
    
    tokio::spawn(async move {
        sched_clone.run(heap_clone, seq_clone, submit_rx, cancel_rx).await;
    });

    // Agora usa o sched original
    sched.submit("kernel_init", Priority::RealTime, 50).await;
    sched.submit("user_task", Priority::Normal, 20).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    sched.report().await;
        }
