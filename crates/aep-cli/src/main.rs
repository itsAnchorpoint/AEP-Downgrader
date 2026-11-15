//! `aep` — command-line front end and test bench for `aep-core`.
//!
//! Commands (argument parsing is done by hand; no dependencies):
//!
//! ```text
//! aep tree <file>        print the chunk tree, one node per line
//! aep roundtrip <file>   parse, serialize, compare with the input
//! ```

use aep_core::riff::{Node, parse};
use std::io::Write;
use std::process::ExitCode;

const USAGE: &str = "usage: aep <tree|roundtrip> <file>";

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

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    match args[0].as_str() {
        "tree" => tree(&args[1]),
        "roundtrip" => roundtrip(&args[1]),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
