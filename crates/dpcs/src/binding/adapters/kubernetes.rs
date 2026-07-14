//! Kubernetes Job / CronJob scaffold adapter (experimental).

use crate::diagnostics::ValidationReport;
use crate::plan::PipelinePlan;

use super::common::{semantics_file, yaml_file, PlanView};
use super::{BindContext, OrchestratorAdapter};
use crate::binding::artifact::{BindingFile, BindingTarget};

/// Kubernetes binding adapter (experimental).
pub struct KubernetesAdapter;

impl OrchestratorAdapter for KubernetesAdapter {
    fn target(&self) -> BindingTarget {
        BindingTarget::Kubernetes
    }

    fn translate(
        &self,
        plan: &PipelinePlan,
        ctx: &BindContext<'_>,
    ) -> Result<Vec<BindingFile>, ValidationReport> {
        let view = PlanView::new(plan, ctx);
        let names = view.unique_k8s_names(&[view.contract_id()]);
        let name = names
            .get(view.contract_id())
            .cloned()
            .unwrap_or_else(|| PlanView::k8s_name(view.contract_id()));
        let step_order_csv = view.step_order().join(",");
        let edges_csv = view
            .dependency_edges()
            .iter()
            .map(|e| format!("{}->{}", e.from, e.to))
            .collect::<Vec<_>>()
            .join(",");

        let configmap = format!(
            "\
# DPCS Kubernetes scaffold (experimental)
apiVersion: v1
kind: ConfigMap
metadata:
  name: {name}-plan
  labels:
    app.kubernetes.io/managed-by: dpcs
    dpcs.io/contract-id: \"{contract_label}\"
    dpcs.io/contract-version: \"{version_label}\"
    dpcs.io/profile: \"{profile_label}\"
data:
  contractId: \"{contract_id}\"
  contractVersion: \"{contract_version}\"
  profileIdentity: \"{profile}\"
  dpcsVersion: \"{dpcs_version}\"
  stepOrder: \"{step_order}\"
  dependencyEdges: \"{edges}\"
  planSummary: |
{plan_summary}
",
            name = name,
            contract_label = PlanView::k8s_name(view.contract_id()),
            version_label = PlanView::k8s_name(view.contract_version()),
            profile_label = PlanView::k8s_name(view.profile_identity()),
            contract_id = PlanView::yaml_string(view.contract_id()),
            contract_version = PlanView::yaml_string(view.contract_version()),
            profile = PlanView::yaml_string(view.profile_identity()),
            dpcs_version = PlanView::yaml_string(&view.plan.dpcs_version),
            step_order = PlanView::yaml_string(&step_order_csv),
            edges = PlanView::yaml_string(&edges_csv),
            plan_summary = indent_block(&plan_summary_yaml(&view), 4),
        );

        let job_or_cron = if let Some(cron) = view.primary_cron() {
            cronjob_yaml(&view, &name, &names, cron)
        } else {
            job_yaml(&view, &name, &names)
        };

        Ok(vec![
            yaml_file("pipeline-configmap.yaml", configmap),
            yaml_file(
                if view.primary_cron().is_some() {
                    "cronjob.yaml"
                } else {
                    "job.yaml"
                },
                job_or_cron,
            ),
            semantics_file(&view),
        ])
    }
}

fn indent_block(text: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{pad}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn plan_summary_yaml(view: &PlanView<'_>) -> String {
    let mut lines = vec![
        format!(
            "contractId: \"{}\"",
            PlanView::yaml_string(view.contract_id())
        ),
        "steps:".to_owned(),
    ];
    for step_id in view.step_order() {
        let step = view.step(step_id);
        let step_type = step.map(|s| s.step_type.as_str()).unwrap_or("unknown");
        let contract = step
            .and_then(|s| s.contract_ref.as_deref().or(s.transform_ref.as_deref()))
            .unwrap_or("-");
        lines.push(format!("  - id: \"{}\"", PlanView::yaml_string(step_id)));
        lines.push(format!(
            "    type: \"{}\"",
            PlanView::yaml_string(step_type)
        ));
        lines.push(format!(
            "    contractRef: \"{}\"",
            PlanView::yaml_string(contract)
        ));
    }
    if !view.quality_gates().is_empty() {
        lines.push("qualityGates:".to_owned());
        for gate in view.quality_gates() {
            lines.push(format!("  - \"{}\"", PlanView::yaml_string(&gate.id)));
        }
    }
    if !view.failure_semantics().is_empty() {
        lines.push("failureSemantics:".to_owned());
        for failure in view.failure_semantics() {
            lines.push(format!("  - \"{}\"", PlanView::yaml_string(&failure.id)));
        }
    }
    lines.join("\n")
}

fn container_block(name: &str, step_id: &str, contract_id: &str) -> String {
    format!(
        "\
        - name: {name}
          image: example.com/dpcs-step:scaffold
          command: [\"/bin/true\"]
          env:
            - name: DPCS_STEP_ID
              value: \"{step}\"
            - name: DPCS_CONTRACT_ID
              value: \"{contract}\"
",
        name = name,
        step = PlanView::yaml_string(step_id),
        contract = PlanView::yaml_string(contract_id),
    )
}

/// Encode steps as initContainers + one main container so they run in order
/// (never as peer containers, which Kubernetes starts in parallel).
fn ordered_pod_spec(
    view: &PlanView<'_>,
    names: &std::collections::BTreeMap<String, String>,
    pipeline_name: &str,
) -> String {
    let order = view.step_order();
    if order.is_empty() {
        return format!(
            "\
      restartPolicy: Never
      containers:
{}",
            container_block(&format!("{pipeline_name}-noop"), "noop", view.contract_id())
        );
    }
    if order.len() == 1 {
        let step_id = &order[0];
        return format!(
            "\
      restartPolicy: Never
      containers:
{}",
            container_block(&names[step_id], step_id, view.contract_id())
        );
    }

    let mut init = String::new();
    for step_id in &order[..order.len() - 1] {
        init.push_str(&container_block(
            &names[step_id],
            step_id,
            view.contract_id(),
        ));
    }
    let last = &order[order.len() - 1];
    format!(
        "\
      restartPolicy: Never
      initContainers:
{init}      containers:
{}",
        container_block(&names[last], last, view.contract_id())
    )
}

fn job_yaml(
    view: &PlanView<'_>,
    name: &str,
    names: &std::collections::BTreeMap<String, String>,
) -> String {
    format!(
        "\
# DPCS Kubernetes Job scaffold (experimental)
# Steps run sequentially via initContainers + main container (topology linearized by step_order).
apiVersion: batch/v1
kind: Job
metadata:
  name: {name}
  labels:
    app.kubernetes.io/managed-by: dpcs
    dpcs.io/contract-id: \"{label}\"
spec:
  template:
    metadata:
      labels:
        dpcs.io/contract-id: \"{label}\"
    spec:
{spec}
",
        name = name,
        label = PlanView::k8s_name(view.contract_id()),
        spec = ordered_pod_spec(view, names, name),
    )
}

fn cronjob_yaml(
    view: &PlanView<'_>,
    name: &str,
    names: &std::collections::BTreeMap<String, String>,
    cron: &str,
) -> String {
    let timezone_line = view
        .primary_timezone()
        .map(|tz| format!("  timeZone: \"{}\"\n", PlanView::yaml_string(tz)))
        .unwrap_or_default();
    format!(
        "\
# DPCS Kubernetes CronJob scaffold (experimental)
# Steps run sequentially via initContainers + main container (topology linearized by step_order).
apiVersion: batch/v1
kind: CronJob
metadata:
  name: {name}
  labels:
    app.kubernetes.io/managed-by: dpcs
    dpcs.io/contract-id: \"{label}\"
spec:
  schedule: \"{cron}\"
{timezone}  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      template:
        metadata:
          labels:
            dpcs.io/contract-id: \"{label}\"
        spec:
{spec}
",
        name = name,
        label = PlanView::k8s_name(view.contract_id()),
        cron = PlanView::yaml_string(cron),
        timezone = timezone_line,
        spec = ordered_pod_spec(view, names, name),
    )
}
