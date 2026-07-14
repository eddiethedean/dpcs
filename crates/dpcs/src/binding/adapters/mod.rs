//! Orchestrator binding adapters.

mod airflow;
mod common;
mod dagster;
mod kubernetes;
mod prefect;
mod temporal;

pub(crate) use common::validate_relative_path;

use crate::diagnostics::ValidationReport;
use crate::plan::PipelinePlan;

use self::airflow::AirflowAdapter;
use self::dagster::DagsterAdapter;
use self::kubernetes::KubernetesAdapter;
use self::prefect::PrefectAdapter;
use self::temporal::TemporalAdapter;
use crate::binding::artifact::{BindingFile, BindingTarget};
use crate::binding::framework::BindContext;

/// Internal adapter contract for translating a plan into platform artifacts.
pub(crate) trait OrchestratorAdapter {
    fn target(&self) -> BindingTarget;

    fn translate(
        &self,
        plan: &PipelinePlan,
        ctx: &BindContext<'_>,
    ) -> Result<Vec<BindingFile>, ValidationReport>;
}

/// Resolve the adapter for a binding target.
pub(crate) fn adapter_for(target: BindingTarget) -> &'static dyn OrchestratorAdapter {
    let adapter: &'static dyn OrchestratorAdapter = match target {
        BindingTarget::Airflow => &AirflowAdapter,
        BindingTarget::Dagster => &DagsterAdapter,
        BindingTarget::Prefect => &PrefectAdapter,
        BindingTarget::Temporal => &TemporalAdapter,
        BindingTarget::Kubernetes => &KubernetesAdapter,
    };
    debug_assert_eq!(adapter.target(), target);
    adapter
}
