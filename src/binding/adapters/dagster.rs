//! Dagster Definitions scaffold adapter.

use crate::diagnostics::ValidationReport;
use crate::plan::PipelinePlan;

use super::common::{python_file, PlanView};
use super::{BindContext, OrchestratorAdapter};
use crate::binding::artifact::{BindingFile, BindingTarget};

/// Dagster binding adapter.
pub struct DagsterAdapter;

impl OrchestratorAdapter for DagsterAdapter {
    fn target(&self) -> BindingTarget {
        BindingTarget::Dagster
    }

    fn translate(
        &self,
        plan: &PipelinePlan,
        ctx: &BindContext<'_>,
    ) -> Result<Vec<BindingFile>, ValidationReport> {
        let view = PlanView::new(plan, ctx);
        let job_name = PlanView::python_ident(view.contract_id());

        let mut body = String::new();
        body.push_str(&view.header_comment("Dagster"));
        body.push('\n');
        body.push_str(
            "\
from dagster import Definitions, OpExecutionContext, graph, job, op

",
        );

        for step_id in view.step_order() {
            let fn_name = PlanView::python_ident(step_id);
            let step = view.step(step_id);
            let contract = step
                .and_then(|s| s.contract_ref.as_deref().or(s.transform_ref.as_deref()))
                .unwrap_or("-");
            let step_type = step.map(|s| s.step_type.as_str()).unwrap_or("unknown");
            body.push_str("@op(name=\"");
            body.push_str(&PlanView::py_string(step_id));
            body.push_str("\", tags={\"dpcs.contractRef\": \"");
            body.push_str(&PlanView::py_string(contract));
            body.push_str("\", \"dpcs.stepType\": \"");
            body.push_str(&PlanView::py_string(step_type));
            body.push_str(
                "\"})
def ",
            );
            body.push_str(&fn_name);
            body.push_str(
                "_op(context: OpExecutionContext) -> None:
    context.log.info(\"DPCS step ",
            );
            body.push_str(&PlanView::py_string(step_id));
            body.push_str("\")\n\n");
        }

        body.push_str("@graph(name=\"");
        body.push_str(&PlanView::py_string(&job_name));
        body.push_str("_graph\")\ndef ");
        body.push_str(&job_name);
        body.push_str("_graph():\n");

        if view.step_order().is_empty() {
            body.push_str("    pass\n");
        } else if view.dependency_edges().is_empty() {
            let mut prev: Option<String> = None;
            for step_id in view.step_order() {
                let fn_name = PlanView::python_ident(step_id);
                body.push_str("    ");
                body.push_str(&fn_name);
                body.push_str("_result = ");
                body.push_str(&fn_name);
                body.push_str("_op()");
                if let Some(p) = &prev {
                    body.push_str("  # after ");
                    body.push_str(p);
                }
                body.push('\n');
                prev = Some(fn_name);
            }
            // Explicit linear deps via graph API notes in comments; sequential calls
            // preserve step_order when edges are absent.
        } else {
            for step_id in view.step_order() {
                let fn_name = PlanView::python_ident(step_id);
                body.push_str("    ");
                body.push_str(&fn_name);
                body.push_str("_result = ");
                body.push_str(&fn_name);
                body.push_str("_op()\n");
            }
            for edge in view.dependency_edges() {
                body.push_str("    # dependency ");
                body.push_str(&edge.from);
                body.push_str(" -> ");
                body.push_str(&edge.to);
                body.push('\n');
            }
        }

        body.push('\n');
        body.push_str(&job_name);
        body.push_str("_job = ");
        body.push_str(&job_name);
        body.push_str("_graph.to_job(name=\"");
        body.push_str(&PlanView::py_string(&job_name));
        body.push_str("\")\n\ndefs = Definitions(jobs=[");
        body.push_str(&job_name);
        body.push_str("_job])\n");

        Ok(vec![python_file("definitions.py", body)])
    }
}
