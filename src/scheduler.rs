use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    RealTime = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Idle = 4,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: u64,
    pub name: String,
    pub priority: Priority,
    pub ticks_left: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HeapEntry {
    pub priority: Priority,
    pub seq: u64,
    pub task_id: u64,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    other
        .priority
        .cmp(&self.priority)
        .then_with(|| other.seq.cmp(&self.seq))
    }

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    
}

let task = Task {
    id,
    name,
    priority,
    ticks_left: ticks,
};
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            ready_heap: Mutex::new(BinaryHeap::new()),
            let entry = HeapEntry {
    priority,
    seq: seq_val,
    task_id: id,
};
        drop(id_guard);

        let task = Task { id, name, priority, ticks_left: ticks };

        let mut seq_guard = self.seq.lock().await;
        let seq_val = *seq_guard;
        *seq_guard += 1;
        drop(seq_guard);
        let entry = HeapEntry {
         priority,
         seq: seq_val,
         task_id: id,
      };

     drop(id_guard);
    }

    pub async fn pop(&self) -> Option<Task> {
        let mut heap_guard = self.ready_heap.lock().await;
        let entry = heap_guard.pop()?; // ← BinaryHeap::pop já faz heapify
        drop(heap_guard);

        let mut tasks_guard = self.tasks.lock().await;
        tasks_guard.remove(&entry.task_id)
    }
  pub async fn run(self: Arc<Self>, mut rx: mpsc::Receiver<Task>) {
        while let Some(task) = rx.recv().await {
            self.submit(task.name, task.priority, task.ticks_left).await;
        }
    }

pub async fn report(&self) {
    let tasks = self.tasks.lock().await;
    let heap = self.ready_heap.lock().await;
   println!("Scheduler report: {} tasks, {} ready", tasks.len(), heap.len());
}
}
}
