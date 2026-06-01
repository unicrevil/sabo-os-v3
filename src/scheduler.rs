use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    RealTime = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

impl Priority {
    pub fn as_u8(self) -> u8 { self as u8 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: u64,
    pub name: String,
    pub priority: Priority,
    pub state: TaskState,
    pub cpu_budget_ms: u64,
    pub cpu_used_ms: u64,
    pub deadline: Option<Instant>,
}

impl Task {
    pub fn new(id: u64, name: &str, priority: Priority, budget_ms: u64) -> Self {
        Self {
            id,
            name: name.to_string(),
            priority,
            state: TaskState::Ready,
            cpu_budget_ms: budget_ms,
            cpu_used_ms: 0,
            deadline: None,
        }
    }

    pub fn remaining_budget(&self) -> u64 {
        self.cpu_budget_ms.saturating_sub(self.cpu_used_ms)
    }

    pub fn is_overdue(&self) -> bool {
        match self.deadline {
            Some(d) => Instant::now() > d,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HeapEntry {
    pub priority: u8,
    pub seq: u64,
    pub task_id: u64,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Scheduler {
    tasks: Arc<Mutex<Vec<Task>>>,
    ready_heap: Arc<Mutex<BinaryHeap<HeapEntry>>>,
    seq: Arc<Mutex<u64>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            ready_heap: Arc::new(Mutex::new(BinaryHeap::new())),
            seq: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn run(
        &self,
        ready_heap: Arc<Mutex<BinaryHeap<HeapEntry>>>,
        seq: Arc<Mutex<u64>>,
        mut submit_rx: mpsc::Receiver<Task>,
        mut cancel_rx: mpsc::Receiver<u64>,
    ) {
        loop {
            tokio::select! {
                Some(mut task) = submit_rx.recv() => {
                    let s = {
                        let mut sq = seq.lock().unwrap();
                        *sq += 1;
                        *sq
                    };

                    let entry = HeapEntry {
                        priority: task.priority.as_u8(),
                        seq: s,
                        task_id: task.id,
                    };

                    task.state = TaskState::Ready;
                    self.tasks.lock().unwrap().push(task);
                    ready_heap.lock().unwrap().push(entry);

                    eprintln!("[SCHED] task enqueued, heap_size={}",
                        ready_heap.lock().unwrap().len());
                }

                Some(cancel_id) = cancel_rx.recv() => {
                    let mut ts = self.tasks.lock().unwrap();
                    if let Some(t) = ts.iter_mut().find(|t| t.id == cancel_id) {
                        t.state = TaskState::Cancelled;
                        eprintln!("[SCHED] task {} '{}' cancelled", t.id, t.name);
                    }
                }

                _ = sleep(Duration::from_millis(1)) => {
                    Self::dispatch_tick(&self.tasks, &ready_heap);
                }
            }
        }
    }

    fn dispatch_tick(
        tasks: &Arc<Mutex<Vec<Task>>>,
        ready_heap: &Arc<Mutex<BinaryHeap<HeapEntry>>>,
    ) {
        let entry = {
            let mut heap = ready_heap.lock().unwrap();
            heap.pop()
        };

        if let Some(e) = entry {
            let mut ts = tasks.lock().unwrap();
            if let Some(task) = ts.iter_mut().find(|t| t.id == e.task_id) {
                if task.state == TaskState::Cancelled {
                    return;
                }
                if task.is_overdue() {
                    eprintln!("[SCHED] DEADLINE MISS task {} '{}'", task.id, task.name);
                    task.state = TaskState::Completed;
                    return;
                }

                let quantum = task.remaining_budget().min(10);
                task.cpu_used_ms += quantum;
                task.state = TaskState::Running;

                eprintln!(
                    "[SCHED] running task {} '{}' pri={:?} used={}ms budget={}ms",
                    task.id, task.name, task.priority,
                    task.cpu_used_ms, task.cpu_budget_ms
                );

                if task.cpu_used_ms >= task.cpu_budget_ms {
                    task.state = TaskState::Completed;
                    eprintln!("[SCHED] task {} '{}' COMPLETED", task.id, task.name);
                } else {
                    task.state = TaskState::Ready;
                    drop(ts);
                }
            }
        }
    }

    pub fn report(&self) {
        let ts = self.tasks.lock().unwrap();
        println!("\n╔══════════╗");
        println!("║        SABO OS v3 — Task Report          ║");
        println!("╠══════════╣");
        for t in ts.iter() {
            println!(
                "|| [{:>4}] {:<16} pri={:?} {:?}",
                t.id, t.name, t.priority, t.state
            );
        }
        println!("╚═══════════\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_creation() {
        let t = Task::new(1, "test_task", Priority::High, 100);
        assert_eq!(t.state, TaskState::Ready);
        assert_eq!(t.remaining_budget(), 100);
        assert!(!t.is_overdue());
    }

    #[tokio::test]
    async fn test_heap_ordering() {
        let mut heap = BinaryHeap::new();
        heap.push(HeapEntry { priority: Priority::Low as u8,      seq: 1, task_id: 10 });
        heap.push(HeapEntry { priority: Priority::RealTime as u8, seq: 2, task_id: 20 });
        heap.push(HeapEntry { priority: Priority::Normal as u8,   seq: 3, task_id: 30 });

        let top = heap.pop().unwrap();
        assert_eq!(top.task_id, 20);
    }

    #[tokio::test]
    async fn test_deadline_miss() {
        let mut t = Task::new(1, "urgent", Priority::RealTime, 50);
        t.deadline = Some(Instant::now() - Duration::from_secs(1));
        assert!(t.is_overdue());
    }
}
