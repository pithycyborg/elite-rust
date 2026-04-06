#!/usr/bin/env -S cargo +nightly -Zscript

/* Rusted-RAT — Single-file Rust memory search & patch tool
 * Zero dependencies. Wildcard search. Multi-pattern + live patching.
 * Built for people who like their tools sharp, fast, and self-contained.
 * 100% free for you to use.
 * Join the author's AI newsletter: PithyCyborg.com
 */

use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::process;

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4 MiB
const MAX_REGION_SIZE: usize = 500_000_000; // 500 MB limit (adjustable)

/// RAII Guard to ensure we detach from the process even if we panic
struct PtraceGuard(i32);
impl PtraceGuard {
    fn attach(pid: i32) -> io::Result<Self> {
        unsafe {
            if libc::ptrace(libc::PTRACE_ATTACH, pid, 0, 0) != 0 {
                return Err(io::Error::last_os_error());
            }
            let mut status = 0i32;
            libc::waitpid(pid, &mut status, 0);
        }
        Ok(PtraceGuard(pid))
    }
}

impl Drop for PtraceGuard {
    fn drop(&mut self) {
        unsafe {
            libc::ptrace(libc::PTRACE_DETACH, self.0, 0, 0);
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.iter().any(|a| a == "--help") {
        print_help(&args[0]);
        return Ok(());
    }

    let mut pid_val: Option<i32> = None;
    let mut search_patterns: Vec<(Vec<u8>, Vec<bool>)> = Vec::new();
    let mut replace_pat: Option<Vec<u8>> = None;
    let mut verbose = false;
    let mut dump_to_file = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--pid" => {
                pid_val = Some(args.get(i + 1).expect("Missing PID").parse().expect("Invalid PID"));
                i += 2;
            }
            "--pattern" => {
                search_patterns.push(parse_hex(args.get(i + 1).expect("Missing hex string")));
                i += 2;
            }
            "--replace" => {
                replace_pat = Some(parse_hex(args.get(i + 1).expect("Missing replace hex")).0);
                i += 2;
            }
            "--verbose" => {
                verbose = true;
                i += 1;
            }
            "--dump" => {
                dump_to_file = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let pid = pid_val.expect("❌ No PID specified. Use --pid <pid>");
    if search_patterns.is_empty() {
        eprintln!("❌ No search pattern specified. Use --pattern <hex>");
        process::exit(1);
    }

    // Validation
    if let Some(ref rep) = replace_pat {
        for (pat, _) in &search_patterns {
            if rep.len() != pat.len() {
                eprintln!("❌ Error: Replacement length ({}) must match pattern length ({})", rep.len(), pat.len());
                process::exit(1);
            }
        }
    }

    println!("⚡ Attaching to PID {}...", pid);
    let _guard = PtraceGuard::attach(pid).map_err(|e| {
        eprintln!("❌ Attach failed: {}. (Try sudo or check yama/ptrace_scope)", e);
        process::exit(1);
    })?;
    println!("✅ Process stopped and attached.");

    let mem_path = format!("/proc/{}/mem", pid);
    let mut mem_file = OpenOptions::new()
        .read(true)
        .write(replace_pat.is_some())
        .open(&mem_path)?;

    let maps = fs::read_to_string(format!("/proc/{}/maps", pid))?;
    let mut matches_found = 0usize;
    let mut region_count = 0usize;

    let mut dump_file = if dump_to_file {
        Some(std::fs::File::create("matches.bin")?)
    } else {
        None
    };

    for line in maps.lines() {
        // Only scan readable, non-special regions
        if !line.contains('r') || line.contains("---p") { continue; }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let range: Vec<&str> = parts[0].split('-').collect();
        let start = usize::from_str_radix(range[0], 16).unwrap_or(0);
        let end = usize::from_str_radix(range[1], 16).unwrap_or(0);
        let region_size = end - start;

        if end <= start { continue; }
        if region_size > MAX_REGION_SIZE {
            if verbose { println!("⚠️ Skipping massive region ({} MB): {}", region_size / 1_000_000, line); }
            continue;
        }

        region_count += 1;
        if verbose { println!("🔍 Scanning: 0x{:X}-0x{:X} | {}", start, end, parts.get(5).unwrap_or(&"anonymous")); }

        let mut offset = 0usize;
        while offset < region_size {
            let chunk_len = CHUNK_SIZE.min(region_size - offset);
            let mut buffer = vec![0u8; chunk_len];

            let addr = (start + offset) as u64;
            if mem_file.seek(SeekFrom::Start(addr)).is_ok() {
                if let Ok(read) = mem_file.read(&mut buffer) {
                    if read == 0 { break; }
                    let actual = &buffer[..read];

                    if let Some(ref mut df) = dump_file { let _ = df.write_all(actual); }

                    for (pat_idx, (search_pat, search_mask)) in search_patterns.iter().enumerate() {
                        let pat_len = search_pat.len();
                        for (i, window) in actual.windows(pat_len).enumerate() {
                            let is_match = window.iter().zip(search_pat).zip(search_mask)
                                .all(|((&b, &p), &m)| !m || b == p);

                            if is_match {
                                let target = start + offset + i;
                                matches_found += 1;
                                println!("🎯 Match #{} [Pat {}] at 0x{:X}", matches_found, pat_idx + 1, target);

                                if let Some(ref replacement) = replace_pat {
                                    mem_file.seek(SeekFrom::Start(target as u64))?;
                                    if let Err(e) = mem_file.write_all(replacement) {
                                        eprintln!("  ⚠️ Patch failed: {}", e);
                                    } else if verbose {
                                        println!("  ✅ Patched.");
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let min_pat_len = search_patterns.iter().map(|(p, _)| p.len()).min().unwrap_or(1);
            offset += if chunk_len > min_pat_len { chunk_len - min_pat_len + 1 } else { chunk_len };
        }
    }

    println!("✅ Done. Scanned {} regions, found {} match(es).", region_count, matches_found);
    Ok(())
}

fn parse_hex(input: &str) -> (Vec<u8>, Vec<bool>) {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.len() % 2 != 0 {
        eprintln!("❌ Hex error: Pattern '{}' must have an even number of characters.", input);
        process::exit(1);
    }
    let mut bytes = Vec::new();
    let mut mask = Vec::new();

    for i in (0..cleaned.len()).step_by(2) {
        let pair = &cleaned[i..i+2];
        if pair == "??" {
            bytes.push(0);
            mask.push(false);
        } else {
            bytes.push(u8::from_str_radix(pair, 16).expect("Invalid hex character"));
            mask.push(true);
        }
    }
    (bytes, mask)
}

fn print_help(name: &str) {
    println!("🚀 Rusted-RAT v3.5 | Memory Search & Patch");
    println!("Usage: {} --pid <pid> --pattern <hex> [OPTIONS]", name);
    println!("\nOptions:");
    println!("  --pid <pid>       Target process ID");
    println!("  --pattern <hex>   Hex search (?? = wildcard). Can be used multiple times.");
    println!("  --replace <hex>   Replacement bytes (must match pattern length)");
    println!("  --verbose         Detailed logging");
    println!("  --dump            Full memory dump of scanned areas to matches.bin");
}
