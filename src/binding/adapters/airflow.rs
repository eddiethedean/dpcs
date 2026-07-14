//! Airflow DAG scaffold adapter.

use crate::diagnostics::ValidationReport;
use crate::plan::PipelinePlan;

use super::common::{python_file, PlanView};
use super::{BindContext, OrchestratorAdapter};
use crate::binding::artifact::{BindingFile, BindingTarget};

/// Apache Airflow binding adapter.
pub struct AirflowAdapter;

impl OrchestratorAdapter for AirflowAdapter {
    fn target(&self) -> BindingTarget {
        BindingTarget::Airflow
    }

    fn translate(
        &self,
        plan: &PipelinePlan,
        ctx: &BindContext<'_>,
    ) -> Result<Vec<BindingFile>, ValidationReport> {
        let view = PlanView::new(plan, ctx);
        let dag_id = PlanView::python_ident(view.contract_id());
        let file_stem = PlanView::python_ident(view.contract_id());
        let schedule = view
            .primary_cron()
            .map(|c| format!("\"{}\"", PlanView::py_string(c)))
            .unwrap_or_else(|| "None".to_owned());
        let timezone = view
            .primary_timezone()
            .map(|tz| format!("\"{}\"", PlanView::py_string(tz)))
            .unwrap_or_else(|| "\"UTC\"".to_owned());

        let mut body = String::new();
        body.push_str(&view.header_comment("Airflow"));
        body.push('\n');
        body.push_str(
            "\
from datetime import datetime

from airflow import DAG
from airflow.operators.empty import EmptyOperator

with DAG(
    dag_id=\"",
        );
        body.push_str(&PlanView::py_string(&dag_id));
        body.push_str(
            "\",
    description=\"DPCS scaffold for ",
        );
        body.push_str(&PlanView::py_string(view.contract_id()));
        body.push_str(
            "\",
    schedule=",
        );
        body.push_str(&schedule);
        body.push_str(
            ",
    start_date=datetime(2026, 1, 1),
    catchup=False,
    tags=[\"dpcs\", \"",
        );
        body.push_str(&PlanView::py_string(view.profile_identity()));
        body.push_str(
            "\"],
    default_args={
        \"owner\": \"dpcs\",
        \"depends_on_past\": False,
    },
) as dag:
    dag.timezone = ",
        );
        body.push_str(&timezone);
        body.push('\n');

        for step_id in view.step_order() {
            let var = PlanView::python_ident(step_id);
            let step = view.step(step_id);
            let contract = step
                .and_then(|s| s.contract_ref.as_deref().or(s.transform_ref.as_deref()))
                .unwrap_or("-");
            let step_type = step.map(|s| s.step_type.as_str()).unwrap_or("unknown");
            body.push_str("    ");
            body.push_str(&var);
            body.push_str(" = EmptyOperator(\n");
            body.push_str("        task_id=\"");
            body.push_str(&PlanView::py_string(step_id));
            body.push_str("\",\n");
            body.push_str("        doc_md=\"type=");
            body.push_str(&PlanView::py_string(step_type));
            body.push_str(" contractRef=");
            body.push_str(&PlanView::py_string(contract));
            body.push_str("\",\n");
            body.push_str("    )\n");
        }

        if view.dependency_edges().is_empty() {
            // Linear chain from step_order when no explicit edges.
            let order = view.step_order();
            for window in order.windows(2) {
                let from = PlanView::python_ident(&window[0]);
                let to = PlanView::python_ident(&window[1]);
                body.push_str("    ");
                body.push_str(&from);
                body.push_str(" >> ");
                body.push_str(&to);
                body.push('\n');
            }
        } else {
            for edge in view.dependency_edges() {
                let from = PlanView::python_ident(&edge.from);
                let to = PlanView::python_ident(&edge.to);
                body.push_str("    ");
                body.push_str(&from);
                body.push_str(" >> ");
                body.push_str(&to);
                body.push('\n');
            }
        }

        // Quality gates / failure semantics as documented metadata tasks comments already
        // appear in the header; keep EmptyOperators only for steps.

        Ok(vec![python_file(&format!("dags/{file_stem}.py"), body)])
    }
}
