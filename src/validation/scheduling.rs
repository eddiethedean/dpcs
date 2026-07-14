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
                if !earliest.trim().is_empty()
                    && !latest.trim().is_empty()
                    && earliest.trim() > latest.trim()
                {
                    report.push(
                        Diagnostic::error(
                            "DPCS-SCH-006",
                            categories::SCHEDULING,
                            "scheduling earliest constraint must not be after latest",
                        )
                        .with_object_ref(format!("{object_ref}.constraints"))
                        .with_remediation(
                            "Ensure constraints.earliest is lexicographically <= constraints.latest for comparable timestamps",
                        ),
                    );
                }
            }
        }
    }

    report
}
