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
    pub ticks: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HeapEntry {
    pub priority: Priority,
    pub seq: u64,
    pub task_id: u64,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Menor priority = maior prioridade. Se empatar, menor seq ganha
        other.priority.cmp(&self.priority)
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Scheduler {
    pub tasks: Mutex<HashMap<u64, Task>>,
    pub ready_heap: Mutex<BinaryHeap<HeapEntry>>,
    pub seq: Mutex<u64>,
    next_id: Mutex<u64>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            ready_heap: Mutex::new(BinaryHeap::new()),
            seq: Mutex::new(0),
            next_id: Mutex::new(1),
        }
    }

    pub async fn submit(&self, name: &str, priority: Priority, ticks: u32) {
        let mut id_guard = self.next_id.lock().await;
        let id = *id_guard;
        *id_guard += 1;
        drop(id_guard);

        let task = Task {
            id,
            name: name.to_string(),
            priority,
            ticks,
        };

        let mut seq_guard = self.seq.lock().await;
        let seq_val = *seq_guard;
        *seq_guard += 1;
        drop(seq_guard);

        let entry = HeapEntry {
            priority,
            seq: seq_val,
            task_id: id,
        };

        self.tasks.lock().await.insert(id, task);
        self.ready_heap.lock().await.push(entry);
    }

    pub async fn run(
        self: Arc<Self>,  // <-- BALA DE PRATA MATA E0382
        heap: Arc<Mutex<BinaryHeap<HeapEntry>>>,
        seq: Arc<Mutex<u64>>,
        mut submit_rx: mpsc::Receiver<Task>,
        mut cancel_rx: mpsc::Receiver<u64>,
    ) {
        loop {
            tokio::select! {
                Some(task) = submit_rx.recv() => {
                    let mut seq_guard = seq.lock().await;
                    let seq_val = *seq_guard;
                    *seq_guard += 1;
                    drop(seq_guard);

                    let entry = HeapEntry {
                        priority: task.priority,
                        seq: seq_val,
                        task_id: task.id,
                    };

                    self.tasks.lock().await.insert(task.id, task);
                    heap.lock().await.push(entry);
                }
                Some(id) = cancel_rx.recv() => {
                    self.tasks.lock().await.remove(&id);
                }
                else => break,
            }
        }
    }

    pub async fn report(&self) {
        let tasks = self.tasks.lock().await;
        let heap = self.ready_heap.lock().await;
        println!("Scheduler report: {} tasks, {} ready", tasks.len(), heap.len());
    }
                }
