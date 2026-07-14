//! Airflow DAG scaffold adapter.

use crate::diagnostics::ValidationReport;
use crate::plan::PipelinePlan;

use super::common::{python_file, semantics_file, PlanView};
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
        let idents = view.unique_python_idents(&[view.contract_id()]);
        let dag_id = idents
            .get(view.contract_id())
            .cloned()
            .unwrap_or_else(|| PlanView::python_ident(view.contract_id()));
        let file_stem = dag_id.clone();
        let schedule = view
            .primary_cron()
            .map(|c| format!("\"{}\"", PlanView::py_string(c)))
            .unwrap_or_else(|| "None".to_owned());

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
",
        );
        if let Some(tz) = view.primary_timezone() {
            body.push_str("    # scheduleTimezone: ");
            body.push_str(&PlanView::py_string(tz));
            body.push_str(" (configure via Airflow timetable / timezone-aware start_date)\n");
        }

        for step_id in view.step_order() {
            let var = &idents[step_id];
            let step = view.step(step_id);
            let contract = step
                .and_then(|s| s.contract_ref.as_deref().or(s.transform_ref.as_deref()))
                .unwrap_or("-");
            let step_type = step.map(|s| s.step_type.as_str()).unwrap_or("unknown");
            body.push_str("    ");
            body.push_str(var);
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

        // Only wire declared dependency edges — never invent edges from step_order.
        for edge in view.dependency_edges() {
            let from = &idents[&edge.from];
            let to = &idents[&edge.to];
            body.push_str("    ");
            body.push_str(from);
            body.push_str(" >> ");
            body.push_str(to);
            body.push('\n');
        }

        Ok(vec![
            python_file(&format!("dags/{file_stem}.py"), body),
            semantics_file(&view),
        ])
    }
}
