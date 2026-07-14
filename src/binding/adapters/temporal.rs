//! Temporal workflow stub adapter (experimental).

use crate::diagnostics::ValidationReport;
use crate::plan::PipelinePlan;

use super::common::{python_file, PlanView};
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
        let workflow_name = PlanView::python_ident(view.contract_id());

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
            let fn_name = PlanView::python_ident(step_id);
            let step = view.step(step_id);
            let contract = step
                .and_then(|s| s.contract_ref.as_deref().or(s.transform_ref.as_deref()))
                .unwrap_or("-");
            body.push_str("@activity.defn(name=\"");
            body.push_str(&PlanView::py_string(step_id));
            body.push_str(
                "\")
async def ",
            );
            body.push_str(&fn_name);
            body.push_str("_activity() -> None:\n    \"\"\"DPCS step ");
            body.push_str(&PlanView::py_string(step_id));
            body.push_str(" contractRef=");
            body.push_str(&PlanView::py_string(contract));
            body.push_str(".\"\"\"\n    return None\n\n");
        }

        body.push_str("@workflow.defn(name=\"");
        body.push_str(&PlanView::py_string(view.contract_id()));
        body.push_str("\")\nclass ");
        // Class names shouldn't have leading underscore weirdness - capitalize
        let class_name = {
            let ident = PlanView::python_ident(view.contract_id());
            let mut chars = ident.chars();
            match chars.next() {
                None => "PipelineWorkflow".to_owned(),
                Some(c) => {
                    let mut s = c.to_ascii_uppercase().to_string();
                    s.push_str(chars.as_str());
                    if !s.ends_with("Workflow") {
                        s.push_str("Workflow");
                    }
                    s
                }
            }
        };
        body.push_str(&class_name);
        body.push_str(
            ":
    @workflow.run
    async def run(self) -> None:
",
        );

        if view.step_order().is_empty() {
            body.push_str("        return None\n");
        } else {
            body.push_str(
                "        # Sequential execution preserves step_order; edges documented below.\n",
            );
            for step_id in view.step_order() {
                let fn_name = PlanView::python_ident(step_id);
                body.push_str("        await workflow.execute_activity(\n            ");
                body.push_str(&fn_name);
                body.push_str("_activity,\n            start_to_close_timeout=timedelta(minutes=30),\n        )\n");
            }
            for edge in view.dependency_edges() {
                body.push_str("        # dependency ");
                body.push_str(&PlanView::py_string(&edge.from));
                body.push_str(" -> ");
                body.push_str(&PlanView::py_string(&edge.to));
                body.push('\n');
            }
        }

        // Silence unused variable if we named workflow_name but used class
        let _ = workflow_name;

        Ok(vec![python_file("workflow.py", body)])
    }
}
