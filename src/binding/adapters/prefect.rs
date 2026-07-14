//! Prefect flow scaffold adapter.

use crate::diagnostics::ValidationReport;
use crate::plan::PipelinePlan;

use super::common::{python_file, PlanView};
use super::{BindContext, OrchestratorAdapter};
use crate::binding::artifact::{BindingFile, BindingTarget};

/// Prefect binding adapter.
pub struct PrefectAdapter;

impl OrchestratorAdapter for PrefectAdapter {
    fn target(&self) -> BindingTarget {
        BindingTarget::Prefect
    }

    fn translate(
        &self,
        plan: &PipelinePlan,
        ctx: &BindContext<'_>,
    ) -> Result<Vec<BindingFile>, ValidationReport> {
        let view = PlanView::new(plan, ctx);
        let flow_name = PlanView::python_ident(view.contract_id());

        let mut body = String::new();
        body.push_str(&view.header_comment("Prefect"));
        body.push('\n');
        body.push_str("from prefect import flow, task\n\n");

        for step_id in view.step_order() {
            let fn_name = PlanView::python_ident(step_id);
            let step = view.step(step_id);
            let contract = step
                .and_then(|s| s.contract_ref.as_deref().or(s.transform_ref.as_deref()))
                .unwrap_or("-");
            body.push_str("@task(name=\"");
            body.push_str(&PlanView::py_string(step_id));
            body.push_str("\", tags=[\"");
            body.push_str(&PlanView::py_string(contract));
            body.push_str(
                "\"])
def ",
            );
            body.push_str(&fn_name);
            body.push_str("_task() -> None:\n    \"\"\"DPCS step ");
            body.push_str(&PlanView::py_string(step_id));
            body.push_str(".\"\"\"\n    pass\n\n");
        }

        body.push_str("@flow(name=\"");
        body.push_str(&PlanView::py_string(view.contract_id()));
        body.push_str("\")\ndef ");
        body.push_str(&flow_name);
        body.push_str("_flow() -> None:\n");
        if view.step_order().is_empty() {
            body.push_str("    pass\n");
        } else {
            body.push_str("    \"\"\"Preserves DPCS step order and dependencies.\"\"\"\n");
            for step_id in view.step_order() {
                let fn_name = PlanView::python_ident(step_id);
                body.push_str("    ");
                body.push_str(&fn_name);
                body.push_str("_task()\n");
            }
            for edge in view.dependency_edges() {
                body.push_str("    # dependency ");
                body.push_str(&PlanView::py_string(&edge.from));
                body.push_str(" -> ");
                body.push_str(&PlanView::py_string(&edge.to));
                body.push('\n');
            }
        }

        if let Some(cron) = view.primary_cron() {
            body.push_str(
                "\n# SchedulingIntent cron (wire externally or via Prefect deployments):\n",
            );
            body.push_str("# schedule: \"");
            body.push_str(&PlanView::py_string(cron));
            body.push_str("\"\n");
            if let Some(tz) = view.primary_timezone() {
                body.push_str("# timezone: \"");
                body.push_str(&PlanView::py_string(tz));
                body.push_str("\"\n");
            }
        }

        body.push_str("\nif __name__ == \"__main__\":\n    ");
        body.push_str(&flow_name);
        body.push_str("_flow()\n");

        Ok(vec![python_file("flow.py", body)])
    }
}
