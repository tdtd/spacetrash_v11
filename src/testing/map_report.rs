use std::fmt::Write;

/// Collects named results for a map assertion run.
///
/// The report keeps all results so a failing test can print the complete
/// map autopsy instead of stopping after the first failed condition.
#[derive(Debug, Default)]
pub struct MapReport {
    entries: Vec<MapReportEntry>,
}

#[derive(Debug)]
struct MapReportEntry {
    passed: bool,
    name: String,
    detail: String,
}

impl MapReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pass(&mut self, name: impl Into<String>) {
        self.entries.push(MapReportEntry {
            passed: true,
            name: name.into(),
            detail: String::new(),
        });
    }

    pub fn fail(&mut self, name: impl Into<String>, detail: impl Into<String>) {
        self.entries.push(MapReportEntry {
            passed: false,
            name: name.into(),
            detail: detail.into(),
        });
    }

    pub fn has_failures(&self) -> bool {
        self.entries.iter().any(|entry| !entry.passed)
    }

    pub fn format_summary(&self) -> String {
        let mut out = String::new();

        for entry in &self.entries {
            let status = if entry.passed { "PASS" } else { "FAIL" };
            let _ = writeln!(
                out,
                "[{status}] {}{}",
                entry.name,
                if entry.detail.is_empty() {
                    String::new()
                } else {
                    format!(" — {}", entry.detail)
                }
            );
        }

        out
    }
}
