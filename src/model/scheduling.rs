//! Scheduling intent model (SPEC Ch 11).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::ExtensionMap;

/// Declared scheduling intent for a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingIntent {
    /// Optional stable identifier for this intent declaration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Scheduling mode (required).
    pub mode: SchedulingMode,
    /// Optional cron-like schedule expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    /// Optional timezone identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Execution frequency where applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    /// Execution windows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub windows: Vec<String>,
    /// Blackout periods.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blackouts: Vec<String>,
    /// Execution deadlines.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deadlines: Vec<String>,
    /// Event declarations for event-driven modes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<SchedulingEvent>,
    /// Timing and concurrency constraints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<SchedulingConstraints>,
    /// Applicable execution policies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Scheduling mode for a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulingMode {
    /// Manual initiation.
    Manual,
    /// On-demand initiation.
    OnDemand,
    /// Time-based scheduled execution.
    Scheduled,
    /// Event-driven execution.
    EventDriven,
    /// Streaming execution intent.
    Streaming,
    /// Continuous execution intent.
    Continuous,
    /// Implementation-defined extension mode.
    Extension(String),
}

impl SchedulingMode {
    /// Returns the wire-form mode string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Manual => "manual",
            Self::OnDemand => "onDemand",
            Self::Scheduled => "scheduled",
            Self::EventDriven => "eventDriven",
            Self::Streaming => "streaming",
            Self::Continuous => "continuous",
            Self::Extension(value) => value.as_str(),
        }
    }

    /// Returns whether this is the scheduled (time-based) mode.
    pub fn is_scheduled(&self) -> bool {
        matches!(self, Self::Scheduled)
    }

    /// Returns whether this is the event-driven mode.
    pub fn is_event_driven(&self) -> bool {
        matches!(self, Self::EventDriven)
    }
}

impl Serialize for SchedulingMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SchedulingMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        let normalized: String = value
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect();
        Ok(match normalized.as_str() {
            "manual" => Self::Manual,
            "ondemand" => Self::OnDemand,
            "scheduled" => Self::Scheduled,
            "eventdriven" => Self::EventDriven,
            "streaming" => Self::Streaming,
            "continuous" => Self::Continuous,
            _ => Self::Extension(value),
        })
    }
}

/// Logical event that may initiate pipeline execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingEvent {
    /// Event identity.
    pub id: String,
    /// Event source.
    pub source: String,
    /// Triggering conditions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Timing and concurrency constraints for scheduling intent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingConstraints {
    /// Earliest execution time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub earliest: Option<String>,
    /// Latest execution time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest: Option<String>,
    /// Execution deadline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// Concurrency limitations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<String>,
    /// Ordering constraints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordering: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
