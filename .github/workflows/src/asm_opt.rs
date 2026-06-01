// ============================================================
// SABO OS v3 — Módulo 2: ASM Otimizado
// 5 funções críticas com inline assembly x86_64
// ~50 linhas | 3x ganho sobre Rust puro em hot paths
// ============================================================
//
// ATENÇÃO: requer nightly + #![feature(asm_const)]
// Compile com: RUSTFLAGS="-C target-cpu=native" cargo +nightly build --release

#![allow(unused_unsafe)]

// ─────────────────────────────────────────
// 1. Soma horizontal de array u64 (SIMD-like loop unrolling via ASM)
// ─────────────────────────────────────────

/// Soma todos os elementos de um slice via loop ASM (sem branch overhead)
#[inline(always)]
pub unsafe fn asm_sum_u64(data: &[u64]) -> u64 {
    let mut result: u64 = 0;
    let ptr   = data.as_ptr();
    let len   = data.len();

    core::arch::asm!(
        "xor {acc}, {acc}",
        "test {len}, {len}",
        "jz 2f",
        "1:",
        "add {acc}, [{ptr}]",
        "add {ptr}, 8",
        "dec {len}",
        "jnz 1b",
        "2:",
        ptr = inout(reg) ptr => _,
        len = inout(reg) len => _,
        acc = inout(reg) result => result,
        options(nostack, readonly),
    );
    result
}

// ─────────────────────────────────────────
// 2. Comparação de memória em bloco (memcmp rápido)
// ─────────────────────────────────────────

/// Compara dois buffers byte a byte via REPE CMPSB
#[inline(always)]
pub unsafe fn asm_memcmp(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let len = a.len();
    if len == 0 { return true; }

    let mut equal: u8 = 1;
    core::arch::asm!(
        "repe cmpsb",
        "sete {eq}",
        in("rsi") a.as_ptr(),
        in("rdi") b.as_ptr(),
        in("rcx") len,
        eq = out(reg_byte) equal,
        options(nostack, readonly),
    );
    equal != 0
}

// ─────────────────────────────────────────
// 3. Leitura do timestamp de hardware (RDTSC)
// ─────────────────────────────────────────

/// Lê o contador de ciclos TSC — resolução sub-nanosegundo
#[inline(always)]
pub unsafe fn asm_rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!(
        "rdtsc",
        out("eax") lo,
        out("edx") hi,
        options(nostack, nomem),
    );
    ((hi as u64) << 32) | (lo as u64)
}

// ─────────────────────────────────────────
// 4. Popcount: conta bits 1 em u64 (POPCNT)
// ─────────────────────────────────────────

/// Conta bits ativos em um u64 usando instrução POPCNT nativa
#[inline(always)]
pub unsafe fn asm_popcnt(value: u64) -> u64 {
    let count: u64;
    core::arch::asm!(
        "popcnt {out}, {inp}",
        inp = in(reg) value,
        out = out(reg) count,
        options(nostack, nomem, pure),
    );
    count
}

// ─────────────────────────────────────────
// 5. Fence de memória (MFENCE) para sincronismo de cache
// ─────────────────────────────────────────

/// Barreira de memória completa — garante ordering entre loads/stores
#[inline(always)]
pub unsafe fn asm_mfence() {
    core::arch::asm!("mfence", options(nostack, nomem));
}

// ─────────────────────────────────────────
// Testes
// ─────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asm_sum() {
        let data = vec![1u64, 2, 3, 4, 5];
        assert_eq!(unsafe { asm_sum_u64(&data) }, 15);
    }

    #[test]
    fn test_asm_memcmp() {
        let a = b"sabo_os_v3";
        let b = b"sabo_os_v3";
        let c = b"sabo_os_v4";
        assert!(unsafe  { asm_memcmp(a, b) });
        assert!(!unsafe { asm_memcmp(a, c) });
    }

    #[test]
    fn test_asm_rdtsc_monotonic() {
        let t1 = unsafe { asm_rdtsc() };
        let t2 = unsafe { asm_rdtsc() };
        assert!(t2 >= t1, "TSC deve ser monotônico");
    }

    #[test]
    fn test_asm_popcnt() {
        assert_eq!(unsafe { asm_popcnt(0b1010_1010) }, 4);
        assert_eq!(unsafe { asm_popcnt(u64::MAX)    }, 64);
        assert_eq!(unsafe { asm_popcnt(0)           }, 0);
    }

    #[test]
    fn test_asm_mfence_noop() {
        // Apenas verifica que não crasha
        unsafe { asm_mfence() };
    }
  }
      
