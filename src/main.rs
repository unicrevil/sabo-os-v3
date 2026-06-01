// ============================================================
// SABO OS v3 — main.rs
// Entry point: wires scheduler + asm + benchmark + logger
// ============================================================

#![feature(asm_const)]

mod scheduler;
mod asm_opt;
mod benchmark;
mod demo_logger;

use demo_logger::{Logger, LogLevel, print_banner};
use benchmark::{BenchRunner, bench_array_sum_rust, bench_fibonacci_rust,
                bench_heap_alloc_rust, bench_hashmap_rust, print_comparison_report};
use scheduler::{Scheduler, Priority};

#[tokio::main]
async fn main() {
    print_banner();

    let log = Logger::new("SABO/main", LogLevel::Debug);
    log.info("Sistema iniciando...");

    // ── Scheduler ────────────────────────────────────────────
    log.info("Inicializando Tokio Scheduler");
    let (sched, submit_rx, cancel_rx) = Scheduler::new();

    let tasks   = sched.tasks.clone();
    let heap    = sched.ready_heap.clone();
    let seq     = sched.seq.clone();

    tokio::spawn(Scheduler::run_dispatch_loop(tasks, heap, seq, submit_rx, cancel_rx));

    sched.submit("kernel_init",   Priority::RealTime, 50).await;
    sched.submit("net_driver",    Priority::High,     100).await;
    sched.submit("bench_runner",  Priority::Normal,   200).await;
    sched.submit("log_flusher",   Priority::Low,      80).await;
    sched.submit("idle_gc",       Priority::Idle,     500).await;

    log.info("5 tarefas submetidas ao scheduler");
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    sched.report();

    // ── Benchmarks ───────────────────────────────────────────
    log.info("Iniciando suite de benchmarks (warmup=100, iter=1000)");

    let mut runner = BenchRunner::new(100, 1_000);

    runner.run("array_sum",  "Rust+ASM", || bench_array_sum_rust(10_000));
    runner.run("fibonacci",  "Rust+ASM", || bench_fibonacci_rust(40));
    runner.run("heap_alloc", "Rust+ASM", || bench_heap_alloc_rust(100) as u64);
    runner.run("hashmap",    "Rust+ASM", || bench_hashmap_rust(500));

    log.info("Benchmarks concluídos");
    print_comparison_report(runner.results());

    log.info("SABO OS v3 — shutdown limpo. Até a próxima. 🦀");
}
