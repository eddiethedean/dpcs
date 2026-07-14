//! Scheduling intent validation phase (SPEC Ch 11).

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate scheduling mode, events, and constraint consistency.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();

    for (index, intent) in contract.scheduling.iter().enumerate() {
        let object_ref = format!("scheduling[{index}]");

        if intent.mode.as_str().trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-SCH-001",
                    categories::SCHEDULING,
                    "scheduling intent must declare a non-empty mode",
                )
                .with_object_ref(format!("{object_ref}.mode")),
            );
        }

        if intent.mode.is_scheduled() {
            let has_frequency = intent
                .frequency
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty());
            let has_cron = intent
                .cron
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty());
            if !has_frequency && !has_cron {
                report.push(
                    Diagnostic::error(
                        "DPCS-SCH-002",
                        categories::SCHEDULING,
                        "scheduled mode requires frequency or cron",
                    )
                    .with_object_ref(object_ref.clone())
                    .with_remediation("Set scheduling[].frequency or scheduling[].cron"),
                );
            }
        }

        if intent.mode.is_event_driven() && intent.events.is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-SCH-003",
                    categories::SCHEDULING,
                    "eventDriven mode requires at least one event declaration",
                )
                .with_object_ref(format!("{object_ref}.events"))
                .with_remediation("Declare scheduling[].events with id and source"),
            );
        }

        for (event_index, event) in intent.events.iter().enumerate() {
            let event_ref = format!("{object_ref}.events[{event_index}]");
            if event.id.trim().is_empty() {
                report.push(
                    Diagnostic::error(
                        "DPCS-SCH-004",
                        categories::SCHEDULING,
                        "scheduling event must declare a non-empty identity",
                    )
                    .with_object_ref(format!("{event_ref}.id")),
                );
            }
            if event.source.trim().is_empty() {
                report.push(
                    Diagnostic::error(
                        "DPCS-SCH-005",
                        categories::SCHEDULING,
                        "scheduling event must declare a non-empty source",
                    )
                    .with_object_ref(format!("{event_ref}.source")),
                );
            }
        }

        if let Some(constraints) = &intent.constraints {
            if let (Some(earliest), Some(latest)) = (&constraints.earliest, &constraints.latest) {
                let earliest = earliest.trim();
                let latest = latest.trim();
                if !earliest.is_empty() && !latest.is_empty() {
                    match (
                        is_comparable_iso8601(earliest),
                        is_comparable_iso8601(latest),
                    ) {
                        (true, true) if earliest > latest => {
                            report.push(
                                Diagnostic::error(
                                    "DPCS-SCH-006",
                                    categories::SCHEDULING,
                                    "scheduling earliest constraint must not be after latest",
                                )
                                .with_object_ref(format!("{object_ref}.constraints"))
                                .with_remediation(
                                    "Use comparable RFC3339/ISO-8601 timestamps with earliest <= latest",
                                ),
                            );
                        }
                        (true, true) => {}
                        _ => {
                            report.push(
                                Diagnostic::warning(
                                    "DPCS-SCH-007",
                                    categories::SCHEDULING,
                                    "scheduling earliest/latest constraints are not comparable ISO-8601 timestamps",
                                )
                                .with_object_ref(format!("{object_ref}.constraints"))
                                .with_remediation(
                                    "Use RFC3339 timestamps such as 2026-01-01T00:00:00Z for timing consistency checks",
                                ),
                            );
                        }
                    }
                }
            }
        }
    }

    report
}

/// Returns true for zero-padded RFC3339/ISO-8601 timestamps that are lexicographically ordered.
fn is_comparable_iso8601(value: &str) -> bool {
    // YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS±HH:MM
    let bytes = value.as_bytes();
    if bytes.len() < 20 {
        return false;
    }
    let date_ok = bytes
        .get(0..4)
        .is_some_and(|b| b.iter().all(u8::is_ascii_digit))
        && bytes.get(4) == Some(&b'-')
        && bytes
            .get(5..7)
            .is_some_and(|b| b.iter().all(u8::is_ascii_digit))
        && bytes.get(7) == Some(&b'-')
        && bytes
            .get(8..10)
            .is_some_and(|b| b.iter().all(u8::is_ascii_digit))
        && bytes.get(10) == Some(&b'T')
        && bytes
            .get(11..13)
            .is_some_and(|b| b.iter().all(u8::is_ascii_digit))
        && bytes.get(13) == Some(&b':')
        && bytes
            .get(14..16)
            .is_some_and(|b| b.iter().all(u8::is_ascii_digit))
        && bytes.get(16) == Some(&b':')
        && bytes
            .get(17..19)
            .is_some_and(|b| b.iter().all(u8::is_ascii_digit));
    if !date_ok {
        return false;
    }
    match bytes.get(19) {
        Some(b'Z') if bytes.len() == 20 => true,
        Some(b'+') | Some(b'-') if bytes.len() == 25 => {
            bytes
                .get(20..22)
                .is_some_and(|b| b.iter().all(u8::is_ascii_digit))
                && bytes.get(22) == Some(&b':')
                && bytes
                    .get(23..25)
                    .is_some_and(|b| b.iter().all(u8::is_ascii_digit))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_comparable_iso8601;

    #[test]
    fn recognizes_rfc3339_timestamps() {
        assert!(is_comparable_iso8601("2026-01-01T00:00:00Z"));
        assert!(is_comparable_iso8601("2026-12-31T23:59:59+00:00"));
        assert!(!is_comparable_iso8601("9:00"));
        assert!(!is_comparable_iso8601("10:00"));
        assert!(!is_comparable_iso8601("P1D"));
    }
}
