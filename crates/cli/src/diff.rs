use std::path::Path;
use serde::Serialize;
use serde_json::{Value, json};
use crate::theme::Styler;

pub fn handle(args: &[String], styler: &Styler, as_json: bool) -> Result<Value, String> {
    if args.len() >= 2 {
        let path_a = &args[0];
        let path_b = &args[1];
        diff_files(Path::new(path_a), Path::new(path_b), styler, as_json)
    } else {
        // Run git diff on workspace or target
        diff_git(args.first().map(String::as_str), styler, as_json)
    }
}

pub fn diff_files(path_a: &Path, path_b: &Path, styler: &Styler, as_json: bool) -> Result<Value, String> {
    let content_a = std::fs::read_to_string(path_a)
        .map_err(|e| format!("failed to read {}: {e}", path_a.display()))?;
    let content_b = std::fs::read_to_string(path_b)
        .map_err(|e| format!("failed to read {}: {e}", path_b.display()))?;

    let lines_a: Vec<&str> = content_a.lines().collect();
    let lines_b: Vec<&str> = content_b.lines().collect();

    let (additions, deletions, hunks) = compute_diff(&lines_a, &lines_b);

    if as_json {
        return Ok(json!({
            "file_a": path_a.to_string_lossy(),
            "file_b": path_b.to_string_lossy(),
            "additions": additions,
            "deletions": deletions,
            "hunks": hunks,
        }));
    }

    if additions == 0 && deletions == 0 {
        println!("{}", styler.lymph(&format!("✓ Files are identical: {} and {}", path_a.display(), path_b.display())));
        return Ok(json!({ "identical": true }));
    }

    let mut output_lines = Vec::new();
    output_lines.push(format!(
        "--- {}  +++ {}",
        styler.danger(&path_a.to_string_lossy()),
        styler.lymph(&path_b.to_string_lossy())
    ));
    output_lines.push(styler.dim("──────────────────────────────────────────────────────────"));

    for h in &hunks {
        output_lines.push(styler.vein(&format!("@@ -{},{} +{},{} @@", h.old_start, h.old_lines, h.new_start, h.new_lines)));
        for line in &h.lines {
            if line.starts_with('+') {
                output_lines.push(styler.lymph(line));
            } else if line.starts_with('-') {
                output_lines.push(styler.danger(line));
            } else {
                output_lines.push(styler.ink_soft(line));
            }
        }
    }

    output_lines.push(styler.dim("──────────────────────────────────────────────────────────"));
    output_lines.push(format!(
        "Summary: {} additions(+), {} deletions(-)",
        styler.lymph(&additions.to_string()),
        styler.danger(&deletions.to_string())
    ));

    let title = format!("Diff: {} ➜ {}", path_a.file_name().unwrap_or_default().to_string_lossy(), path_b.file_name().unwrap_or_default().to_string_lossy());
    println!("{}", styler.card(&title, &output_lines));

    Ok(json!({
        "additions": additions,
        "deletions": deletions,
    }))
}

pub fn diff_git(target: Option<&str>, styler: &Styler, as_json: bool) -> Result<Value, String> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("diff");
    if let Some(t) = target {
        cmd.arg(t);
    }

    let output = cmd.output().map_err(|e| format!("failed to run git diff: {e}"))?;
    let raw = String::from_utf8_lossy(&output.stdout);

    if raw.trim().is_empty() {
        if as_json {
            return Ok(json!({ "clean": true, "message": "Working directory clean; no diff" }));
        }
        println!("{}", styler.lymph("✓ Working directory clean. No uncommitted diffs found."));
        return Ok(json!({ "clean": true }));
    }

    if as_json {
        return Ok(json!({ "raw": raw }));
    }

    let mut lines = Vec::new();
    for l in raw.lines() {
        if l.starts_with("diff --git") || l.starts_with("index ") {
            lines.push(styler.bold(&styler.ink(l)));
        } else if l.starts_with("--- ") || l.starts_with("+++ ") {
            lines.push(styler.bold(&styler.nerve(l)));
        } else if l.starts_with("@@ ") {
            lines.push(styler.vein(l));
        } else if l.starts_with('+') {
            lines.push(styler.lymph(l));
        } else if l.starts_with('-') {
            lines.push(styler.danger(l));
        } else {
            lines.push(styler.ink_soft(l));
        }
    }

    println!("{}", styler.card("Git Repository Workspace Diff", &lines));
    Ok(json!({ "diff_lines": lines.len() }))
}

#[derive(Debug, Clone, Serialize)]
pub struct Hunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<String>,
}

fn compute_diff(a: &[&str], b: &[&str]) -> (usize, usize, Vec<Hunk>) {
    let mut additions = 0;
    let mut deletions = 0;
    let mut hunks = Vec::new();

    // Simple LCS-based diff calculation
    let m = a.len();
    let n = b.len();

    // If both empty
    if m == 0 && n == 0 {
        return (0, 0, hunks);
    }

    let mut cur_hunk_lines = Vec::new();
    let mut i = 0;
    let mut j = 0;

    while i < m || j < n {
        if i < m && j < n && a[i] == b[j] {
            cur_hunk_lines.push(format!("  {}", a[i]));
            i += 1;
            j += 1;
        } else if j < n && (i >= m || !a[i..].contains(&b[j])) {
            cur_hunk_lines.push(format!("+ {}", b[j]));
            additions += 1;
            j += 1;
        } else if i < m {
            cur_hunk_lines.push(format!("- {}", a[i]));
            deletions += 1;
            i += 1;
        }
    }

    if !cur_hunk_lines.is_empty() {
        hunks.push(Hunk {
            old_start: 1,
            old_lines: m,
            new_start: 1,
            new_lines: n,
            lines: cur_hunk_lines,
        });
    }

    (additions, deletions, hunks)
}
