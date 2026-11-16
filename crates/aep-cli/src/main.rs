//! `aep` — command-line front end and test bench for `aep-core`.
//!
//! Commands (argument parsing is done by hand; no dependencies):
//!
//! ```text
//! aep tree <file>                    print the chunk tree, one node per line
//! aep roundtrip <file>               parse, serialize, compare with the input
//! aep diff [--full] [--no-noise] <a> <b>   structural diff of two files
//! ```

use aep_core::diff::{self, Change};
use aep_core::riff::{Node, parse};
use std::io::Write;
use std::process::ExitCode;

const USAGE: &str = "usage: aep <tree|roundtrip|diff> ...\n\
                     aep tree <file>\n\
                     aep roundtrip <file>\n\
                     aep diff [--full] [--no-noise] <a> <b>";

/// Write to stdout without panicking on I/O errors (e.g. broken pipe).
fn emit(text: &str) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(text.as_bytes());
    let _ = handle.flush();
}

/// Render a four-character code as ASCII, replacing non-printable bytes.
fn fmt_id(b: &[u8; 4]) -> String {
    b.iter()
        .map(|&c| {
            if (0x20..=0x7e).contains(&c) {
                c as char
            } else {
                '?'
            }
        })
        .collect()
}

fn read_and_parse(path: &str) -> Result<(Vec<u8>, aep_core::riff::RiffFile), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let file = parse(&bytes).map_err(|e| format!("parse error in {path}: {e:?}"))?;
    Ok((bytes, file))
}

fn tree(path: &str) -> ExitCode {
    let (_, file) = match read_and_parse(path) {
        Ok(r) => r,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(1);
        }
    };
    let mut out = String::new();
    file.walk(&mut |enclosing, node| print_node(enclosing, node, &mut out));
    emit(&out);
    ExitCode::from(0)
}

fn print_node(enclosing: &[aep_core::riff::FourCc], node: &Node, out: &mut String) {
    // `walk` already visits every node, parents before children, so this prints
    // exactly one line for the node it receives and never descends.
    let indent = "  ".repeat(enclosing.len());
    match node {
        Node::Chunk { id, data } => {
            out.push_str(&format!("{indent}{} {}\n", fmt_id(id), data.len()));
        }
        Node::List { kind, .. } => {
            out.push_str(&format!(
                "{indent}LIST:{} {}\n",
                fmt_id(kind),
                node.data_len()
            ));
        }
        Node::OpaqueList { kind, .. } => {
            out.push_str(&format!(
                "{indent}LIST:{} {} (opaque)\n",
                fmt_id(kind),
                node.data_len()
            ));
        }
    }
}

fn roundtrip(path: &str) -> ExitCode {
    let (original, file) = match read_and_parse(path) {
        Ok(r) => r,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(1);
        }
    };
    let out = file.to_bytes();
    if out != original {
        let i = out
            .iter()
            .zip(original.iter())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| out.len().min(original.len()));
        emit(&format!("MISMATCH at byte {i}\n"));
        return ExitCode::from(1);
    }
    let mut n = 0u64;
    file.walk(&mut |_enclosing, _node| n += 1);
    emit(&format!("OK {} bytes, {} nodes\n", original.len(), n));
    ExitCode::from(0)
}

/// Format an index path for display: indices joined by `.`, or `/` if empty.
fn fmt_index_path(index_path: &[usize]) -> String {
    if index_path.is_empty() {
        "/".to_string()
    } else {
        index_path
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}

/// Format a path for display: `/` if empty, otherwise the path itself.
fn fmt_path(path: &str) -> &str {
    if path.is_empty() { "/" } else { path }
}

/// Format samples as `off:aa>bb` triples in lowercase hex, space-separated.
fn fmt_samples(samples: &[(u64, u8, u8)]) -> String {
    samples
        .iter()
        .map(|&(off, a, b)| format!("{off:x}:{a:02x}>{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn diff_cmd(full: bool, no_noise: bool, path_a: &str, path_b: &str) -> ExitCode {
    let (_, file_a) = match read_and_parse(path_a) {
        Ok(r) => r,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(1);
        }
    };
    let (_, file_b) = match read_and_parse(path_b) {
        Ok(r) => r,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(1);
        }
    };

    let changes = diff::diff(&file_a, &file_b);
    let mut out = String::new();

    let display_changes: Vec<Change> = if no_noise {
        changes
            .iter()
            .filter(|c| !diff::is_save_noise(c))
            .cloned()
            .collect()
    } else {
        changes.clone()
    };
    let noise_hidden = changes.len() - display_changes.len();

    if full {
        for change in &display_changes {
            match change {
                Change::Removed(l) => {
                    let prev_suffix = if l.prev.is_empty() {
                        String::new()
                    } else {
                        format!(" after \"{}\"", l.prev)
                    };
                    out.push_str(&format!(
                        "removed  {}  {} {} {} \"{}\"{}\n",
                        fmt_path(&l.path),
                        l.key,
                        fmt_index_path(&l.index_path),
                        l.len,
                        l.preview,
                        prev_suffix
                    ));
                }
                Change::Added(l) => {
                    let prev_suffix = if l.prev.is_empty() {
                        String::new()
                    } else {
                        format!(" after \"{}\"", l.prev)
                    };
                    out.push_str(&format!(
                        "added    {}  {} {} {} \"{}\"{}\n",
                        fmt_path(&l.path),
                        l.key,
                        fmt_index_path(&l.index_path),
                        l.len,
                        l.preview,
                        prev_suffix
                    ));
                }
                Change::Changed(p) => {
                    let samples = fmt_samples(&p.samples);
                    if samples.is_empty() {
                        out.push_str(&format!(
                            "changed  {}  {} {} {}->{} first={} count={}\n",
                            fmt_path(&p.path),
                            p.key,
                            fmt_index_path(&p.index_path),
                            p.len_a,
                            p.len_b,
                            p.first_diff,
                            p.diff_count
                        ));
                    } else {
                        out.push_str(&format!(
                            "changed  {}  {} {} {}->{} first={} count={} samples={}\n",
                            fmt_path(&p.path),
                            p.key,
                            fmt_index_path(&p.index_path),
                            p.len_a,
                            p.len_b,
                            p.first_diff,
                            p.diff_count,
                            samples
                        ));
                    }
                }
            }
        }
    } else {
        for line in diff::summarize(&display_changes) {
            out.push_str(&line);
            out.push('\n');
        }
    }

    let n = display_changes.len();
    if no_noise && noise_hidden > 0 {
        out.push_str(&format!("{n} changes ({noise_hidden} noise hidden)\n"));
    } else {
        out.push_str(&format!("{n} changes\n"));
    }
    emit(&out);

    if n == 0 && noise_hidden == 0 {
        ExitCode::from(0)
    } else {
        ExitCode::from(1)
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    match args[0].as_str() {
        "tree" => {
            if args.len() != 2 {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            tree(&args[1])
        }
        "roundtrip" => {
            if args.len() != 2 {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            roundtrip(&args[1])
        }
        "diff" => {
            let rest: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
            let mut full = false;
            let mut no_noise = false;
            let mut files: Vec<&str> = Vec::new();
            for arg in rest {
                if arg == "--full" {
                    full = true;
                } else if arg == "--no-noise" {
                    no_noise = true;
                } else {
                    files.push(arg);
                }
            }
            if files.len() != 2 {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            diff_cmd(full, no_noise, files[0], files[1])
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
