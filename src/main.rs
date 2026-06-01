mod scheduler;

use scheduler::{Priority, Scheduler, Task};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let scheduler = Scheduler::new();
    let (_tx, rx) = mpsc::channel::<Task>(100);

    let sched = Arc::new(scheduler);
    let sched_clone = Arc::clone(&sched);

    tokio::spawn(async move {
        sched_clone.run(Arc::clone(&sched_clone), rx).await;
    });

    // Teste rápido
    sched.submit("task1".to_string(), Priority::High, 10).await;
    sched.submit("task2".to_string(), Priority::Normal, 5).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    sched.report().await;

    println!("✅ SABO OS v3 — Sistema funcionando!");
}
