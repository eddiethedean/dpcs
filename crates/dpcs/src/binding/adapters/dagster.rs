//! Dagster Definitions scaffold adapter.

use crate::diagnostics::ValidationReport;
use crate::plan::PipelinePlan;

use super::common::{python_file, semantics_file, PlanView};
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
        let idents = view.unique_python_idents(&[view.contract_id()]);
        let job_name = idents
            .get(view.contract_id())
            .cloned()
            .unwrap_or_else(|| PlanView::python_ident(view.contract_id()));

        let mut body = String::new();
        body.push_str(&view.header_comment("Dagster"));
        body.push('\n');
        body.push_str("from dagster import Definitions, OpExecutionContext, graph, op\n\n");

        for step_id in view.step_order() {
            let fn_name = &idents[step_id];
            let step = view.step(step_id);
            let contract = step
                .and_then(|s| s.contract_ref.as_deref().or(s.transform_ref.as_deref()))
                .unwrap_or("-");
            let step_type = step.map(|s| s.step_type.as_str()).unwrap_or("unknown");
            let preds = view.predecessors(step_id);
            body.push_str("@op(name=\"");
            body.push_str(&PlanView::py_string(step_id));
            body.push_str("\", tags={\"dpcs.contractRef\": \"");
            body.push_str(&PlanView::py_string(contract));
            body.push_str("\", \"dpcs.stepType\": \"");
            body.push_str(&PlanView::py_string(step_type));
            body.push_str("\"})\ndef ");
            body.push_str(fn_name);
            if preds.is_empty() {
                body.push_str("_op(context: OpExecutionContext):\n");
            } else {
                body.push_str("_op(context: OpExecutionContext, upstream):\n");
            }
            body.push_str("    context.log.info(\"DPCS step ");
            body.push_str(&PlanView::py_string(step_id));
            body.push_str("\")\n    return True\n\n");
        }

        body.push_str("@graph(name=\"");
        body.push_str(&PlanView::py_string(&job_name));
        body.push_str("_graph\")\ndef ");
        body.push_str(&job_name);
        body.push_str("_graph():\n");

        if view.step_order().is_empty() {
            body.push_str("    pass\n");
        } else if view.dependency_edges().is_empty() {
            // Independent entry ops — do not invent linear dependencies.
            for step_id in view.step_order() {
                let fn_name = &idents[step_id];
                body.push_str("    ");
                body.push_str(fn_name);
                body.push_str("_result = ");
                body.push_str(fn_name);
                body.push_str("_op()\n");
            }
        } else {
            for step_id in view.step_order() {
                let fn_name = &idents[step_id];
                let preds = view.predecessors(step_id);
                body.push_str("    ");
                body.push_str(fn_name);
                body.push_str("_result = ");
                body.push_str(fn_name);
                body.push_str("_op(");
                if preds.is_empty() {
                    body.push(')');
                } else if preds.len() == 1 {
                    body.push_str(&idents[preds[0]]);
                    body.push_str("_result)");
                } else {
                    body.push('[');
                    for (i, pred) in preds.iter().enumerate() {
                        if i > 0 {
                            body.push_str(", ");
                        }
                        body.push_str(&idents[*pred]);
                        body.push_str("_result");
                    }
                    body.push_str("])");
                }
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

        Ok(vec![
            python_file("definitions.py", body),
            semantics_file(&view),
        ])
    }
}
