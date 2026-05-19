//! Stdout formatters for the read-only modes (`--check`, `--dry-run`,
//! `--audit`). Mutation reports land in a later commit alongside the
//! mutator itself.

#![allow(dead_code)] // additional formatters land with the mutator

use super::discover::{HitDetail, RemovalPlan};

/// Print the audit / dry-run preview. Groups hits by file so users see
/// each path once with all the things uninstall would touch under it.
pub(crate) fn print_audit(plan: &RemovalPlan, header: &str) {
    println!("{header}");
    println!("{}", "=".repeat(header.len()));

    if plan.hits.is_empty() && plan.scan_dir_hits.is_empty() {
        println!("No known ICM residue found.");
        return;
    }

    print_section("Configured locations", &plan.hits);
    if !plan.scan_dir_hits.is_empty() {
        print_section("Project tree references (--scan-dir)", &plan.scan_dir_hits);
    }

    if !plan.processes.is_empty() {
        println!();
        println!("Running `icm serve` processes:");
        for p in &plan.processes {
            println!("  pid={:<6} {}", p.pid, p.cmdline);
        }
    }

    println!();
    println!("Total: {} item(s).", plan.total_hits());
}

fn print_section(title: &str, hits: &[super::discover::LocationHit]) {
    if hits.is_empty() {
        return;
    }
    println!();
    println!("{title}");
    println!("{}", "-".repeat(title.len()));

    // Group by path so multiple hits in the same file collapse together.
    let mut current: Option<&std::path::Path> = None;
    for hit in hits {
        if current != Some(hit.path.as_path()) {
            println!();
            println!(
                "{:<28} {}",
                format!("[{}]", hit.spec_label),
                hit.path.display()
            );
            current = Some(hit.path.as_path());
        }
        match &hit.detail {
            HitDetail::JsonServer { pointer } => {
                println!("  MCP server entry at {pointer}");
            }
            HitDetail::JsonHook { event, command } => {
                println!("  hook {event}: {command}");
            }
            HitDetail::TomlTable { table } => {
                println!("  TOML table {table}");
            }
            HitDetail::YamlBlock { start_line, lines } => {
                println!(
                    "  YAML block at line {start_line} (~{lines} line(s)) — manual review may be needed"
                );
            }
            HitDetail::MarkdownBlock {
                start_line,
                end_line,
                file_will_be_empty,
            } => {
                let tag = if *file_will_be_empty {
                    " (file will be deleted, no other content)"
                } else {
                    ""
                };
                println!("  Markdown block lines {start_line}-{end_line}{tag}");
            }
            HitDetail::OwnedFile { bytes } => {
                println!("  Owned file ({} byte(s)) — will be deleted", bytes);
            }
            HitDetail::DataDir { bytes_total, files } => {
                println!(
                    "  Data directory: {files} file(s), {} byte(s) — kept unless --purge-data",
                    bytes_total
                );
            }
        }
    }
}

/// Brief output for `--check`. Returns the exit code.
pub(crate) fn print_check(plan: &RemovalPlan) -> i32 {
    if plan.is_empty() {
        println!("OK: no known ICM residue found");
        super::exit_codes::CLEAN
    } else {
        println!("FOUND: {} known ICM residue item(s)", plan.total_hits());
        super::exit_codes::CHECK_RESIDUE
    }
}
