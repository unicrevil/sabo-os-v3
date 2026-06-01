// SABO OS v3 - main.rs
mod scheduler;
mod asm_opt;
mod benchmark;
mod demo_logger;

use demo_logger::{Logger, LogLevel, print_banner};
use benchmark::{BenchRunner, bench_array_sum_rust, bench_fibonacci_rust, bench_heap_alloc_rust, bench_hashmap_rust, print_comparison_report};
use scheduler::{Scheduler, Priority, Task};
use tokio::sync::mpsc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    print_banner();
    let log = Logger::new("SABO/main", LogLevel::Debug);
    log.info("Sistema iniciando...");

    log.info("Inicializando Tokio Scheduler");
    let sched = Scheduler::new();
    
    let (submit_tx, submit_rx) = mpsc::channel::<Task>(100);
    let (cancel_tx, cancel_rx) = mpsc::channel::<u64>(100);

    let heap = sched.ready_heap.clone();
    let seq = sched.seq.clone();
    
    tokio::spawn(sched.run(heap, seq, submit_rx, cancel_rx));

    sched.submit("kernel_init",  Priority::RealTime, 50).await;
    sched.submit("net_driver",   Priority::High,     100).await;
    sched.submit("bench_runner", Priority::Normal,   200).await;
    sched.submit("log_flusher",  Priority::Low,      80).await;
    sched.submit("idle_gc",      Priority::Low,      500).await;

    log.info("5 tarefas submetidas ao scheduler");
    sleep(Duration::from_millis(50)).await;
    sched.report();

    log.info("Iniciando suite de benchmarks (warmup=100, iter=1000)");
    let mut runner = BenchRunner::new(100, 1_000);

    runner.run("array_sum",  "Rust+ASM", || bench_array_sum_rust(10_000));
    runner.run("fibonacci",  "Rust+ASM", || bench_fibonacci_rust(40));
    runner.run("heap_alloc", "Rust+ASM", || bench_heap_alloc_rust(100) as u64);
    runner.run("hashmap",    "Rust+ASM", || bench_hashmap_rust(500));
    log.info("Benchmarks concluidos");
    print_comparison_report(runner.results());

    log.info("SABO OS v3 - shutdown limpo. Até a próxima. 🚀");
    }
