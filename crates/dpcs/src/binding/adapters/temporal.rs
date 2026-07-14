//! Temporal workflow stub adapter (experimental).

use crate::diagnostics::ValidationReport;
use crate::plan::PipelinePlan;

use super::common::{python_file, semantics_file, PlanView};
use super::{BindContext, OrchestratorAdapter};
use crate::binding::artifact::{BindingFile, BindingTarget};

/// Temporal binding adapter (experimental).
pub struct TemporalAdapter;

impl OrchestratorAdapter for TemporalAdapter {
    fn target(&self) -> BindingTarget {
        BindingTarget::Temporal
    }

    fn translate(
        &self,
        plan: &PipelinePlan,
        ctx: &BindContext<'_>,
    ) -> Result<Vec<BindingFile>, ValidationReport> {
        let view = PlanView::new(plan, ctx);
        let idents = view.unique_python_idents(&[]);
        let class_name = PlanView::python_class_name(view.contract_id());

        let mut body = String::new();
        body.push_str(&view.header_comment("Temporal (experimental)"));
        body.push('\n');
        body.push_str(
            "\
from datetime import timedelta

from temporalio import activity, workflow

",
        );

        for step_id in view.step_order() {
            let fn_name = &idents[step_id];
            let step = view.step(step_id);
            let contract = step
                .and_then(|s| s.contract_ref.as_deref().or(s.transform_ref.as_deref()))
                .unwrap_or("-");
            body.push_str("@activity.defn(name=\"");
            body.push_str(&PlanView::py_string(step_id));
            body.push_str("\")\nasync def ");
            body.push_str(fn_name);
            body.push_str("_activity() -> None:\n    \"\"\"DPCS step ");
            body.push_str(&PlanView::py_string(step_id));
            body.push_str(" contractRef=");
            body.push_str(&PlanView::py_string(contract));
            body.push_str(".\"\"\"\n    return None\n\n");
        }

        body.push_str("@workflow.defn(name=\"");
        body.push_str(&PlanView::py_string(view.contract_id()));
        body.push_str("\")\nclass ");
        body.push_str(&class_name);
        body.push_str(
            ":
    @workflow.run
    async def run(self) -> None:
",
        );

        if view.step_order().is_empty() {
            body.push_str("        return None\n");
        } else if view.dependency_edges().is_empty() {
            body.push_str(
                "        # Independent activities: sequential awaits are a linearization only.\n",
            );
            for step_id in view.step_order() {
                let fn_name = &idents[step_id];
                body.push_str("        await workflow.execute_activity(\n            ");
                body.push_str(fn_name);
                body.push_str(
                    "_activity,\n            start_to_close_timeout=timedelta(minutes=30),\n        )\n",
                );
            }
        } else {
            body.push_str(
                "        # Await activities in topologic step_order; edges are enforced by sequencing.\n",
            );
            for step_id in view.step_order() {
                let fn_name = &idents[step_id];
                let preds = view.predecessors(step_id);
                if !preds.is_empty() {
                    body.push_str("        # waits for: ");
                    body.push_str(&preds.join(", "));
                    body.push('\n');
                }
                body.push_str("        await workflow.execute_activity(\n            ");
                body.push_str(fn_name);
                body.push_str(
                    "_activity,\n            start_to_close_timeout=timedelta(minutes=30),\n        )\n",
                );
            }
        }

        Ok(vec![
            python_file("workflow.py", body),
            semantics_file(&view),
        ])
    }
}
