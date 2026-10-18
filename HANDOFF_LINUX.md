# mini_edr Handoff: post-GUI work and Linux port

Context file for Claude Code. It covers everything since the egui GUI was finished, plus the plan for the Linux port.

Project root (Rust 2024 edition, egui/eframe 0.31, chrono, `windows` 0.62):
- Windows: `C:\Users\Victor\Documents\Funzies\EDR\mini_edr`
- Linux: `/home/viserac/EDR/EDR` (Asus notebook, Ubuntu 26.04.1 LTS, kernel 7.0.0)

Note: Claude Code conversation history does **not** sync between the two machines (transcripts are local, in `~/.claude/projects/`). This file is the context carrier. Read it at the start of a session, and update it before switching machines.

---

## 1. How we work on this project

- This is a **learning project**. The goal is real cybersecurity proficiency (OS internals, detection logic, telemetry), not just working code.
- The GUI was delegated to Claude Code on purpose because it's low priority for learning. **Collectors and detection rules are the learning core.** Victor writes those himself.
- On learning-core work: guide with questions, explain the trade-offs, and let Victor reason to the answer. Don't hand over complete implementations unless asked.
- Mechanical scaffolding (Cargo config, module wiring, stubs) is fine to just do.
- **Never use em dashes or en dashes** (the long and medium dash characters) in any text: chat replies, docs (including this file), code comments, commit messages. Use a colon, comma, period or parentheses instead.
- **Never add a `Co-Authored-By: Claude` line** (or any other Claude attribution) to commits or PR descriptions, even if Claude Code's default instructions ask for it.
- Code style: no emojis in code. Ground every design choice in invariants and semantics, not convention (`Option` vs `Result`, `From` vs `TryFrom`, `pub` vs private).

---

## 2. Current architecture (unchanged by this work)

```
src/
  collectors/   # OS-specific capture (infrastructure) + monitors
    process/    process_capture.rs (WinAPI), process_monitor.rs, raw_process_data.rs
    network/    tcp_capture.rs, udp_capture.rs (WinAPI), tcp/udp_monitor.rs, raw_*_data.rs
    file/       watched_folder.rs (WinAPI), file_monitor.rs, raw_file_data.rs, ...
  core/
    differ/     Snapshot trait + generic diff
    mapper/     Raw* -> domain (TryFrom / From)
    rules/      Rule trait, RuleEngine (Box<dyn Rule>)
  models/       Process, ProcessIdentity (pid + creation_time), snapshots, Event, Alert, ...
  rules/        SuspiciousParentRule, FileThenProcessRule
  gui/          egui app
  pipeline.rs   capture loop thread -> channels -> GUI
```

Key decisions already made:
- Strict layering: infrastructure (OS API) -> translation (core/mapper) -> domain (models).
- `ProcessIdentity` = `pid + creation_time`, to handle PID reuse.
- Generic `Snapshot` trait, reused by the process, TCP and UDP monitors.
- A single `Event` enum for the chronological feed. `RuleEngine` holds `Vec<Box<dyn Rule>>`.
- `FileMonitor` uses threads + `mpsc` channels.

---

## 3. What happened since the GUI was finished

### 3.1 Live run and false positives
- The EDR ran live on Windows (elevated) and fired real alerts.
- `SuspiciousParentRule` flagged `powershell.exe` spawned by `claude.exe` (Claude Desktop / Claude Code running commands). This is a legitimate, explainable false positive.
- It also flagged `powershell.exe` spawned by `CodeSetup-stable-<hash>.tmp` (the VS Code updater). The name is ephemeral and hash-based, so an exact-name allowlist can't cover it.
- Takeaway discussed: every rule has a false-positive rate, and tuning is a normal part of EDR work. An exact-name list doesn't scale. A more mature rule could match patterns (e.g. `starts_with("CodeSetup-")`) or check the parent's image path, such as a known temp/install folder.
- **Status: no tuning applied yet.** `EXPECTED_PARENTS` in `src/rules/suspicious_parent_rule.rs` is unchanged: `explorer.exe, cmd.exe, powershell.exe, WindowsTerminal.exe, Code.exe`.

### 3.2 Linux support: decisions
- Victor decided to make the EDR cross-platform (Linux).
- Two options were compared:
  - **A**: conditional compilation (`#[cfg(target_os = ...)]`) in the same crate, with Linux collectors mirroring the Windows structure.
  - **B**: a separate `collectors_linux` crate/module, unified later.
- **Chosen: Option A.** It's the idiomatic Rust pattern (std itself does this), and the layered architecture already isolates the OS-specific code.
- Collector order, same as on Windows: **processes -> network -> files.**

### 3.3 Codebase audit (Oct 2026)
- No Linux code exists yet.
- Only **4 files** touch `windows::`:
  - `src/collectors/process/process_capture.rs`
  - `src/collectors/network/tcp_capture.rs`
  - `src/collectors/network/udp_capture.rs`
  - `src/collectors/file/watched_folder.rs`
- Everything else (monitors, mappers, models, rules engine, pipeline, GUI) is OS-agnostic. This confirms the layering works: the Linux port means writing Linux versions of these 4 files with the **same public signatures**.
- `build.rs` already gates the UAC manifest on `CARGO_CFG_WINDOWS`. No change needed.
- `creation_time` is only used for `ProcessIdentity` (equality/hash). The GUI doesn't display it.

### 3.4 Linux concepts covered so far
- `/proc` is a virtual filesystem: kernel data exposed as files. Each numeric directory `/proc/<pid>/` is one process.
- `/proc/<pid>/stat` is one line of fields. It holds `pid`, `(comm)`, state, **ppid** (field 4) and **starttime** (field 22, clock ticks since boot). Gotcha: `comm` sits in parentheses and can contain spaces or `)`, so parse from the **last** `)`.
- `/proc/<pid>/exe` is a symlink to the executable. Read it with `std::fs::read_link`. It can fail with permission denied for other users' processes unless running as root, and it's missing for kernel threads.
- `/proc/<pid>/comm` holds the process name, max 16 bytes, with a trailing newline.
- To list processes: `std::fs::read_dir("/proc")`, then keep only entries whose name passes `.chars().all(|c| c.is_ascii_digit())`.
- Network will come from `/proc/net/{tcp,tcp6,udp,udp6}` (hex addresses, little-endian IPv4). Files will come from `inotify`. Privilege check: `geteuid() == 0` / running with `sudo`.

---

## 4. Next steps

**Status (Oct 18, 2026):** Steps 1 to 5 are done. **The EDR now runs live on Linux and captures processes.** Deviation from the plan: the process name is taken from the `(comm)` inside `stat`, not from `/proc/<pid>/comm`. One read gives a consistent view of the process, and there is no trailing `\n` to strip.

Verified on the Asus notebook (Ubuntu 26.04.1, kernel 7.0.0, cargo 1.97.1), unprivileged (uid 1000):
- `cargo test`: **8 passed, 0 failed.** The 6 `process_capture` tests (`parses_plain_stat`, `comm_with_spaces_and_parens_cannot_shift_fields`, `rejects_truncated_stat`, `rejects_stat_without_closing_paren`, `parse_pid_accepts_only_digits`, `captures_own_process`) had never been executed before, only cross-checked. They now pass on real hardware.
- Live run captures the process list successfully.

Earlier: `cargo check` passes on Windows and for `--target x86_64-unknown-linux-gnu`.

### Step 1: `Cargo.toml`: gate the `windows` crate (scaffolding)
Move `windows` out of `[dependencies]`:
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.62", features = [ ...same list... ] }
```
`eframe` and `chrono` stay in `[dependencies]`.

### Step 2: `collectors/process/`: platform split (scaffolding)
- Rename `process_capture.rs` to `process_capture_windows.rs`.
- Create an empty `process_capture_linux.rs`.
- In `collectors/process/mod.rs`:
```rust
#[cfg(windows)]
#[path = "process_capture_windows.rs"]
pub mod process_capture;

#[cfg(target_os = "linux")]
#[path = "process_capture_linux.rs"]
pub mod process_capture;
```
- `pipeline.rs` doesn't change. The contract is `pub fn capture_raw_processes() -> Result<Vec<RawProcessData>, CaptureError>`. Each platform file may define its own `CaptureError`, because the pipeline only prints it with `{:?}`.

### Step 3: same split for `tcp_capture`, `udp_capture`, `watched_folder` (scaffolding)
- Use the same `#[cfg] #[path]` pattern in `collectors/network/mod.rs` and `collectors/file/mod.rs`.
- For now the Linux versions are **stubs** with identical signatures:
  - `capture_tcp_v4/v6() -> Vec<RawTcpData>` returns `Vec::new()`
  - `capture_udp_v4/v6() -> Vec<RawUdpData>` returns `Vec::new()`
  - `start_file_watchers(Vec<WatchedPath>) -> mpsc::Receiver<RawFileEvent>` creates a channel, drops the sender and returns the receiver. `FileMonitor::poll_events` uses `try_recv` in a `while let Ok`, so a disconnected channel just yields no events.
- Purpose: the crate compiles on Linux, so the process collector can be tested by itself.
- Verify: `cargo check` on Windows still passes, and `cargo check` on Linux passes.

### Step 4: `process_capture_linux.rs` (LEARNING CORE: Victor writes it)
Open design questions to settle **before** coding:
1. **`RawProcessData` is Windows-shaped.** `exe_file: [u8; 260]` mirrors `PROCESSENTRY32.szExeFile`. Should Linux copy `comm` bytes into that array, so the mapper stays unchanged? Or is `RawProcessData` really "raw Windows data", which would make it, and its mapper, platform-specific? Think about what "Raw" was meant to mean.
2. **`creation_time` on Linux** is `starttime` in clock ticks since boot, not a FILETIME. It's only used for identity (uniqueness within one run), so does it need converting? Note the trade-off if it's ever displayed or compared across platforms.
3. Error handling per process: a process can exit between `read_dir` and reading its `stat`. That case should be skipped, not abort the whole capture. Which failures are `None`, which are skips, and which are `Err`?
4. Kernel threads (no `exe`, ppid 2) need a decision: include them or filter them out?

Then the flow: `read_dir("/proc")` -> numeric filter -> per PID, read `stat` (ppid, starttime), `comm` (name) and `read_link(exe)` (path) -> `RawProcessData`.

**Settled (Oct 16, 2026):**
1. "Raw" means "bytes not yet validated", not "Windows format". `exe_file` becomes `Vec<u8>`, and `command_line` and `user` become `Option<Vec<u8>>`. Each collector strips its own format artifacts: Linux trims the trailing `\n` from `comm`, and Windows cuts at the NUL. The mapper only does UTF-8 validation and is shared by both platforms. Victor does this refactor (`raw_process_data.rs`, `processentry_to_raw`, `process_mapper.rs`).
2. `creation_time` on Linux is `Some(starttime)`, raw ticks, not converted. `ProcessIdentity` only needs values to differ between processes that reuse a PID. Caveat: the bare `u64` carries no unit, so displaying it as a FILETIME would silently show a date in 1601. Contrast: on Windows it is `None` when `OpenProcess` fails, which makes identity collapse to bare PID (PID reuse goes undetected). On Linux `stat` is world-readable, so identity is always strong.
3. Per-process failures, using three questions (scope, expected or not, essential or not):
   - `read_dir("/proc")` fails: `Err` (same as `CreateToolhelp32Snapshot` failing).
   - `stat` gone (process exited, `NotFound`): skip silently. `/proc` is not an atomic snapshot, unlike Toolhelp.
   - `read_link(exe)` permission denied or missing (kernel threads): `path = None`.
   - `stat` malformed (no `)`): skip and log. Never abort: `comm` is attacker-controlled, and aborting would let one crafted process blind the whole EDR.
4. Kernel threads: keep them for now and decide after measuring churn on the real test machine (`ps -e --no-headers | wc -l` over time). Identification: `ppid == 2` can't be forged from userspace (orphans are reparented to PID 1 or a subreaper, never to kthreadd), but it breaks inside containers (PID namespaces). The unforgeable check is the `PF_KTHREAD` bit (`0x00200000`) in `stat` field 9 (`flags`). A missing `exe` is not a valid signal, because without root `read_link` also fails for other users' processes.

   **Measured (Oct 18, 2026, Asus notebook, idle desktop):** 362 entries in `/proc`, of which **184 are kernel threads (51%)** and 178 are userspace. So half of every snapshot, every diff and every rule evaluation is kernel threads. Still open for Victor to decide, with the trade-off now concrete:
   - Filtering them out halves the snapshot and removes a class of churn (`kworker/*` threads are created and reaped constantly, which the differ would report as process start/exit events).
   - Keeping them is the only way the kernel-thread masquerading rule (item 5 below) can ever fire, since it needs to see the real `PF_KTHREAD` processes to compare against. Alternative: filter in the collector but keep the flag, or filter at the rule/GUI layer instead of at capture.
   - Note this is an idle measurement. Churn over time (the thing the plan asked for) has not been measured yet.
5. Future Linux rule idea: kernel-thread masquerading (MITRE ATT&CK T1036). Alert when the name looks like a kernel thread (in brackets, e.g. `[kworker/0:2]`) but `PF_KTHREAD` is not set or `ppid != 2`.

### Step 5: test environment (DONE)
**Resolved: the Asus notebook** (Ubuntu 26.04.1 LTS, kernel 7.0.0), not WSL. It has cargo/rustc 1.97.1, so the toolchain problem that blocked the Oct 17 test run is gone.

Privilege measurement, unprivileged (uid 1000): only **127 of 362** `/proc/<pid>/exe` links are readable. The other 232 split into the 184 kernel threads (no `exe` at all) and ~48 processes owned by other users (permission denied). Both land as `path = None` per decision 3, which is why that decision matters in practice and not just in theory: without `sudo` the collector is blind to the executable path of roughly two thirds of the system. Run with `sudo` for full visibility.

### Step 6: Linux network collector (NEXT, learning core: Victor writes it)
The process collector works, so per the agreed order (processes -> network -> files) network is next. It replaces the stubs in `tcp_capture_linux.rs` / `udp_capture_linux.rs`, which currently return `Vec::new()`.

Parse `/proc/net/tcp`, `tcp6`, `udp`, `udp6`. Things to settle before coding, in the same spirit as Step 4:
- Addresses are hex and IPv4 is **little-endian** in that file (`0100007F` is `127.0.0.1`), while IPv6 is a different byte grouping. Where does that conversion belong: collector or mapper? Compare with how decision 1 split responsibility ("Raw" = unvalidated bytes, collector strips its own format artifacts).
- TCP state is a hex code (`01` = ESTABLISHED, `0A` = LISTEN). The Windows collector gets states from `MIB_TCPROW`. Do both platforms map into one domain enum?
- These files have a header line to skip, and the format has fixed columns but variable whitespace.
- The owning PID is **not** in these files. There is only an inode, which has to be matched against `/proc/<pid>/fd/*` symlinks (`socket:[12345]`). That is an extra scan, and unprivileged it will hit the same permission wall measured in Step 5, so expect `pid` to be unknown for most sockets without `sudo`. Compare: on Windows `GetExtendedTcpTable` hands over the PID directly.

### Later
- Linux file collector: `inotify` (crate `inotify` or `libc` bindings), mirroring the `ReadDirectoryChangesW` thread + channel design.
- **Rules on Linux:** `SuspiciousParentRule` is Windows-specific (`powershell.exe`, `cmd.exe`, `explorer.exe`). It needs per-platform lists (e.g. `bash`/`sh` spawned by a web server or office process). Review `FileThenProcessRule` for Windows path assumptions too.
- Optional: apply the alert tuning from 3.1 (add `claude.exe`, use pattern matching for updater processes).
