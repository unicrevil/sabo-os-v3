use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    RealTime,
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Completed,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: u64,
    pub name: String,
    pub priority: Priority,
    pub state: TaskState,
    pub ticks: u32,
}

#[derive(Debug, Clone, Eq)]
pub struct HeapEntry {
    pub priority: Priority,
    pub seq: u64,
    pub task_id: u64,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => other.seq.cmp(&self.seq),
            ord => ord,
        }
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.seq == other.seq
    }
}

pub struct Scheduler {
    pub tasks: Arc<Mutex<Vec<Task>>>,
    pub ready_heap: Arc<Mutex<BinaryHeap<HeapEntry>>>,
    pub seq: Arc<Mutex<u64>>,
    next_id: Arc<Mutex<u64>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            ready_heap: Arc::new(Mutex::new(BinaryHeap::new())),
            seq: Arc::new(Mutex::new(0)),
            next_id: Arc::new(Mutex::new(1)),
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
            state: TaskState::Ready,
            ticks,
        };

        let mut seq_guard = self.seq.lock().await;
        let seq = *seq_guard;
        *seq_guard += 1;
        drop(seq_guard);

        self.tasks.lock().await.push(task);
        self.ready_heap.lock().await.push(HeapEntry {
            priority,
            seq,
            task_id: id,
        });

        println!("[SUBMIT] {} id={} prio={:?}", name, id, priority);
    }

    pub async fn run(
        self,
        heap: Arc<Mutex<BinaryHeap<HeapEntry>>>,
        seq: Arc<Mutex<u64>>,
        mut submit_rx: mpsc::Receiver<Task>,
        mut cancel_rx: mpsc::Receiver<u64>,
    ) {
        loop {
            tokio::select! {
                Some(task) = submit_rx.recv() => {
                    let mut seq_guard = seq.lock().await;
                    let s = *seq_guard;
                    *seq_guard += 1;
                    drop(seq_guard);
                    heap.lock().await.push(HeapEntry {
                        priority: task.priority,
                        seq: s,
                        task_id: task.id,
                    });
                }
                Some(id) = cancel_rx.recv() => {
                    println!("[CANCEL] task id={}", id);
                }
                else => break,
            }
        }
    }

    pub async fn report(&self) {
        let tasks = self.tasks.lock().await;
        println!("\n=== SCHEDULER REPORT ===");
        for t in tasks.iter() {
            println!("Task {}: {} prio={:?} state={:?} ticks={}", t.id, t.name, t.priority, t.state, t.ticks);
        }
    }
    }
