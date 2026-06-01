// ============================================================
// SABO OS v3 — Módulo 4: Demo / Logger
// Terminal bonito com cores ANSI — perfeito pro vídeo de 30s
// ~100 linhas
// ============================================================

use std::time::{SystemTime, UNIX_EPOCH};

// ─────────────────────────────────────────
// Códigos ANSI
// ─────────────────────────────────────────

const RESET:   &str = "\x1b[0m";
const BOLD:    &str = "\x1b[1m";
const DIM:     &str = "\x1b[2m";

const RED:     &str = "\x1b[31m";
const GREEN:   &str = "\x1b[32m";
const YELLOW:  &str = "\x1b[33m";
const BLUE:    &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN:    &str = "\x1b[36m";
const WHITE:   &str = "\x1b[37m";

// ─────────────────────────────────────────
// Níveis de Log
// ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info  = 2,
    Warn  = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    fn label(&self) -> &str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info  => " INFO",
            LogLevel::Warn  => " WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }

    fn color(&self) -> &str {
        match self {
            LogLevel::Trace => DIM,
            LogLevel::Debug => CYAN,
            LogLevel::Info  => GREEN,
            LogLevel::Warn  => YELLOW,
            LogLevel::Error => RED,
            LogLevel::Fatal => MAGENTA,
        }
    }
}

// ─────────────────────────────────────────
// Logger
// ─────────────────────────────────────────

pub struct Logger {
    min_level: LogLevel,
    prefix:    String,
}

impl Logger {
    pub fn new(prefix: &str, min_level: LogLevel) -> Self {
        Self { min_level, prefix: prefix.to_string() }
    }

    fn timestamp() -> String {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("{:013}", ms)
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        if level < self.min_level { return; }

        let ts    = Self::timestamp();
        let color = level.color();
        let label = level.label();

        println!(
            "{DIM}[{ts}]{RESET} {color}{BOLD}[{label}]{RESET} {BLUE}{prefix}{RESET} {WHITE}{msg}{RESET}",
            DIM    = DIM,
            ts     = ts,
            RESET  = RESET,
            color  = color,
            BOLD   = BOLD,
            label  = label,
            BLUE   = BLUE,
            prefix = self.prefix,
            WHITE  = WHITE,
            msg    = msg,
        );
    }

    // Helpers
    pub fn trace(&self, msg: &str) { self.log(LogLevel::Trace, msg); }
    pub fn debug(&self, msg: &str) { self.log(LogLevel::Debug, msg); }
    pub fn info (&self, msg: &str) { self.log(LogLevel::Info,  msg); }
    pub fn warn (&self, msg: &str) { self.log(LogLevel::Warn,  msg); }
    pub fn error(&self, msg: &str) { self.log(LogLevel::Error, msg); }
    pub fn fatal(&self, msg: &str) { self.log(LogLevel::Fatal, msg); }
}

/// Banner de boot do SABO OS v3
pub fn print_banner() {
    println!("{CYAN}{BOLD}", CYAN = CYAN, BOLD = BOLD);
    println!(r"  ███████╗ █████╗ ██████╗  ██████╗      ██████╗ ███████╗");
    println!(r"  ██╔════╝██╔══██╗██╔══██╗██╔═══██╗    ██╔═══██╗██╔════╝");
    println!(r"  ███████╗███████║██████╔╝██║   ██║    ██║   ██║███████╗ ");
    println!(r"  ╚════██║██╔══██║██╔══██╗██║   ██║    ██║   ██║╚════██║ ");
    println!(r"  ███████║██║  ██║██████╔╝╚██████╔╝    ╚██████╔╝███████║ ");
    println!(r"  ╚══════╝╚═╝  ╚═╝╚═════╝  ╚═════╝      ╚═════╝ ╚══════╝");
    println!("{RESET}  {DIM}v3.0.0 | Rust + Tokio + ASM | Benchmark Edition{RESET}\n",
        RESET = RESET, DIM = DIM);
}

// ─────────────────────────────────────────
// Testes
// ─────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Fatal > LogLevel::Info);
        assert!(LogLevel::Trace < LogLevel::Debug);
    }

    #[test]
    fn test_logger_filters() {
        // Não deve crashar com nenhum nível
        let logger = Logger::new("TEST", LogLevel::Warn);
        logger.info("esta linha nao aparece");
        logger.warn("esta sim");
        logger.error("esta tambem");
    }
}
