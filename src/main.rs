mod scheduler;

use scheduler::{HeapEntry, Priority, Scheduler, Task};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let scheduler = Arc::new(Scheduler::new());
    let (tx, rx) = mpsc::channel::<Task>(100);

    let sched_clone = Arc::clone(&scheduler);
    tokio::spawn(async move {
        sched_clone.run(rx).await;
    });

    // Teste rápido
    scheduler.submit("task1".to_string(), Priority::High, 10).await;
    scheduler.submit("task2".to_string(), Priority::Normal, 5).await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    scheduler.report().await;
}
