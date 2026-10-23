//! Bounded reload correlation using a unique development-only load sentinel.
use crate::console::classify::Category;
use anyhow::{Result, ensure};
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) struct Reload {
    pub marker: String,
    started: Option<Instant>,
    dispatch_started: Option<Instant>,
    failures: usize,
    warnings: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Outcome {
    Complete { warnings: usize },
    Failed { diagnostics: usize, warnings: usize },
    TimedOut,
}
impl Reload {
    pub fn new() -> Result<Self> {
        let mut bytes = [0; 16];
        getrandom::fill(&mut bytes).map_err(|_| anyhow::anyhow!("OS entropy unavailable"))?;
        let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        Ok(Self {
            marker: format!("sand_dev_reload_{token}"),
            started: None,
            dispatch_started: None,
            failures: 0,
            warnings: 0,
        })
    }
    /// Correlate logs and arm the completion deadline immediately before the
    /// command write is attempted. A failed write/acknowledgement is uncertain,
    /// so it must still have bounded sentinel tracking. Installation and RCON
    /// authentication happen before this boundary and consume no loading budget.
    pub fn dispatching(&mut self, now: Instant) {
        self.dispatch_started = Some(now);
        self.started = Some(now);
    }

    /// Whether transport failure occurred after command dispatch was attempted.
    pub fn dispatch_attempted(&self) -> bool {
        self.dispatch_started.is_some()
    }

    pub fn includes(&self, received_at: Instant) -> bool {
        self.dispatch_started
            .is_some_and(|start| received_at >= start)
    }

    /// Record dispatch acknowledgement without extending an already-armed
    /// completion deadline or changing the queued-log correlation boundary.
    pub fn dispatched(&mut self, now: Instant) {
        self.started.get_or_insert(now);
    }

    pub fn observe(&mut self, category: Category, message: &str) -> Option<Outcome> {
        match category {
            Category::DatapackError | Category::FatalError => self.failures += 1,
            Category::Warning => self.warnings += 1,
            _ => {}
        }
        // Vanilla renders /say as "[Server] <message>". Matching the complete
        // suffix avoids ordinary diagnostics mentioning the function id.
        if message == format!("[Server] {}", self.marker)
            || message == format!("[Not Secure] [Server] {}", self.marker)
        {
            return Some(if self.failures == 0 {
                Outcome::Complete {
                    warnings: self.warnings,
                }
            } else {
                Outcome::Failed {
                    diagnostics: self.failures,
                    warnings: self.warnings,
                }
            });
        }
        None
    }
    pub fn timeout(&self, now: Instant) -> Option<Outcome> {
        (now.duration_since(self.started?) >= Duration::from_secs(30)).then_some(
            if self.failures == 0 {
                Outcome::TimedOut
            } else {
                Outcome::Failed {
                    diagnostics: self.failures,
                    warnings: self.warnings,
                }
            },
        )
    }
}

pub(super) fn add_marker(stage: &Path, namespace: &str, marker: &str) -> Result<()> {
    let function = stage.join(format!("data/{namespace}/function/{marker}.mcfunction"));
    ensure!(
        !function.exists(),
        "development load marker collides with a pack function"
    );
    std::fs::create_dir_all(function.parent().unwrap())?;
    std::fs::write(function, format!("say {marker}\n"))?;
    let tag = stage.join("data/minecraft/tags/function/load.json");
    let mut json: serde_json::Value = if tag.exists() {
        serde_json::from_slice(&std::fs::read(&tag)?)?
    } else {
        serde_json::json!({"values": []})
    };
    let values = json["values"]
        .as_array_mut()
        .ok_or_else(|| anyhow::anyhow!("invalid load tag values"))?;
    values.push(format!("{namespace}:{marker}").into());
    std::fs::create_dir_all(tag.parent().unwrap())?;
    std::fs::write(tag, serde_json::to_vec(&json)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slow_installation_does_not_spend_the_reload_budget() {
        let before_install = Instant::now();
        let mut reload = Reload::new().unwrap();
        let after_dispatch = before_install + Duration::from_secs(90);
        assert_eq!(
            reload.timeout(after_dispatch),
            None,
            "installation has no loading deadline"
        );
        reload.dispatched(after_dispatch);
        assert_eq!(reload.timeout(after_dispatch), None);
        assert_eq!(
            reload.timeout(after_dispatch + Duration::from_secs(29)),
            None
        );
        assert_eq!(
            reload.timeout(after_dispatch + Duration::from_secs(30)),
            Some(Outcome::TimedOut)
        );
    }

    #[test]
    fn attempted_dispatch_bounds_completion_even_without_acknowledgement() {
        let before_install_and_auth = Instant::now();
        for acknowledged in [false, true] {
            let mut reload = Reload::new().unwrap();
            let attempted = before_install_and_auth + Duration::from_secs(90);
            assert!(!reload.dispatch_attempted());
            assert_eq!(reload.timeout(attempted), None);
            reload.dispatching(attempted);
            assert!(reload.dispatch_attempted());
            assert!(!reload.includes(attempted - Duration::from_nanos(1)));
            assert!(reload.includes(attempted));
            if acknowledged {
                reload.dispatched(attempted + Duration::from_secs(2));
            }
            assert_eq!(reload.timeout(attempted + Duration::from_secs(29)), None);
            assert_eq!(
                reload.timeout(attempted + Duration::from_secs(30)),
                Some(Outcome::TimedOut)
            );
            assert!(
                reload.includes(attempted),
                "ack must not move the queued-log boundary"
            );
        }
    }

    #[test]
    fn vanilla_secure_and_offline_say_markers_use_real_parser_and_classifier() {
        use crate::console::{
            classify::classify,
            log_record::{Stream, parse_line},
        };
        for prefix in ["[Server]", "[Not Secure] [Server]"] {
            let mut reload = Reload::new().unwrap();
            let raw = format!(
                "[12:00:00] [Server thread/INFO]: {prefix} {}",
                reload.marker
            );
            let record = parse_line(&raw);
            let category = classify(Stream::Stdout, &record);
            assert_eq!(
                reload.observe(category, &record.message),
                Some(Outcome::Complete { warnings: 0 })
            );
            for message in [
                format!("[Player] {}", reload.marker),
                format!("{prefix} {} suffix", reload.marker),
                format!("prefix {prefix} {}", reload.marker),
            ] {
                assert_eq!(reload.observe(Category::CommandResponse, &message), None);
            }
        }
    }

    #[test]
    fn quoted_load_errors_do_not_fail_reload_but_server_errors_still_do() {
        use crate::console::{
            classify::classify,
            log_record::{Stream, parse_line},
        };
        for prefix in ["[Server]", "[Not Secure] [Server]", "[Server:"] {
            for actual_error in [false, true] {
                let mut reload = Reload::new().unwrap();
                let suffix = if prefix == "[Server:" { "]" } else { "" };
                let raw = format!(
                    "[12:00:00] [Server thread/INFO]: {prefix} Failed to load function demo:test{suffix}"
                );
                let record = parse_line(&raw);
                let category = classify(Stream::Stdout, &record);
                assert_eq!(category, Category::CommandResponse);
                assert_eq!(reload.observe(category, &record.message), None);
                if actual_error {
                    let record = parse_line(
                        "[12:00:01] [Worker-Main-1/ERROR]: Failed to load function demo:broken",
                    );
                    let category = classify(Stream::Stdout, &record);
                    assert_eq!(category, Category::DatapackError);
                    reload.observe(category, &record.message);
                }
                assert_eq!(
                    reload.observe(
                        Category::CommandResponse,
                        &format!("[Server] {}", reload.marker)
                    ),
                    Some(if actual_error {
                        Outcome::Failed {
                            diagnostics: 1,
                            warnings: 0,
                        }
                    } else {
                        Outcome::Complete { warnings: 0 }
                    })
                );
            }
        }
    }

    #[test]
    fn completion_distinguishes_warnings_errors_and_timeout() {
        let now = Instant::now();
        let mut reload = Reload::new().unwrap();
        assert_eq!(reload.observe(Category::Warning, "warning"), None);
        assert_eq!(reload.observe(Category::Visible, "unrelated"), None);
        assert_eq!(
            reload.observe(
                Category::CommandResponse,
                &format!("[Server] {}", reload.marker)
            ),
            Some(Outcome::Complete { warnings: 1 })
        );
        let mut reload = Reload::new().unwrap();
        reload.observe(Category::DatapackError, "Failed to load function");
        assert_eq!(
            reload.observe(
                Category::CommandResponse,
                &format!("[Server] {}", reload.marker)
            ),
            Some(Outcome::Failed {
                diagnostics: 1,
                warnings: 0
            })
        );
        let mut reload = Reload::new().unwrap();
        reload.dispatched(now);
        assert_eq!(reload.timeout(now + Duration::from_secs(29)), None);
        assert_eq!(
            reload.timeout(now + Duration::from_secs(30)),
            Some(Outcome::TimedOut)
        );
    }
}
