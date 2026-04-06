```markdown
# ⚡ Rusted-RAT v3.5
> The Minimalist Linux Memory Surgeon.

Rusted-RAT is a high-performance, single-file memory scanner and live patcher for Linux. Designed for security researchers and reverse engineers who value speed over bloat.

### Why it's Elite
- Zero-Config: No Cargo.toml required. Run as a script or compile with rustc.
- RAII Safety: Custom Rust guards ensure target processes are always resumed, even on panic.
- Pattern Mastery: Full wildcard support (e.g. 48??89E5??90) for dynamic signatures.
- Smart Scanning: 4 MiB chunking with overlap logic to never miss patterns split across boundaries.
- Forensic Mode: Use --dump to extract scanned regions to matches.bin.

### Quick Start

**Run as a Script (requires Cargo Nightly)**
```bash
chmod +x rusted_rat.rs
sudo ./rusted_rat.rs --pid 1234 --pattern "48??89E5??90"
```

**Compile to a Tiny Binary**
```bash
rustc -O rusted_rat.rs -o rrat
sudo ./rrat --pid 1234 --pattern "4889E5" --replace "909090" --verbose
```

### Troubleshooting

If attach fails, your kernel is likely restricting ptrace. Fix it with:
```bash
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope
```

Warning: Patching live processes can instantly crash the target. Only use on test binaries you control.

### Design Philosophy

Rusted-RAT was born from a frustration with bloated forensic frameworks. Everything wants heavy dependencies or massive environments. This project is a reminder that you don't need a massive ecosystem to build serious tools - just clarity, discipline, and a bit of Rust. Writing it in one source file is an experiment in purity: Simplicity scales.

---

Author: Pithy Cyborg  
Website: https://pithycyborg.com  
Newsletter: https://pithycyborg.substack.com/subscribe  
X/Twitter: https://x.com/pithycyborg  
X/Twitter: https://x.com/mrcomputersci  

License: MIT © 2026 Pithy Cyborg
```
