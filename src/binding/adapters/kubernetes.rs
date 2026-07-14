//! Kubernetes Job / CronJob scaffold adapter (experimental).

use crate::diagnostics::ValidationReport;
use crate::plan::PipelinePlan;

use super::common::{yaml_file, PlanView};
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
        let name = PlanView::k8s_name(view.contract_id());
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
            contract_id = escape_yaml_double(view.contract_id()),
            contract_version = escape_yaml_double(view.contract_version()),
            profile = escape_yaml_double(view.profile_identity()),
            dpcs_version = escape_yaml_double(&view.plan.dpcs_version),
            step_order = escape_yaml_double(&step_order_csv),
            edges = escape_yaml_double(&edges_csv),
            plan_summary = indent_block(&plan_summary_yaml(&view), 4),
        );

        let job_or_cron = if let Some(cron) = view.primary_cron() {
            cronjob_yaml(&view, &name, cron)
        } else {
            job_yaml(&view, &name)
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
        ])
    }
}

fn escape_yaml_double(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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
        format!("contractId: \"{}\"", escape_yaml_double(view.contract_id())),
        "steps:".to_owned(),
    ];
    for step_id in view.step_order() {
        let step = view.step(step_id);
        let step_type = step.map(|s| s.step_type.as_str()).unwrap_or("unknown");
        let contract = step
            .and_then(|s| s.contract_ref.as_deref().or(s.transform_ref.as_deref()))
            .unwrap_or("-");
        lines.push(format!("  - id: \"{}\"", escape_yaml_double(step_id)));
        lines.push(format!("    type: \"{}\"", escape_yaml_double(step_type)));
        lines.push(format!(
            "    contractRef: \"{}\"",
            escape_yaml_double(contract)
        ));
    }
    if !view.quality_gates().is_empty() {
        lines.push("qualityGates:".to_owned());
        for gate in view.quality_gates() {
            lines.push(format!("  - \"{}\"", escape_yaml_double(&gate.id)));
        }
    }
    if !view.failure_semantics().is_empty() {
        lines.push("failureSemantics:".to_owned());
        for failure in view.failure_semantics() {
            lines.push(format!("  - \"{}\"", escape_yaml_double(&failure.id)));
        }
    }
    lines.join("\n")
}

fn container_spec(view: &PlanView<'_>, name: &str) -> String {
    let mut containers = String::new();
    for step_id in view.step_order() {
        let container_name = PlanView::k8s_name(step_id);
        containers.push_str(&format!(
            "\
        - name: {container_name}
          image: example.com/dpcs-step:scaffold
          command: [\"/bin/true\"]
          env:
            - name: DPCS_STEP_ID
              value: \"{step}\"
            - name: DPCS_CONTRACT_ID
              value: \"{contract}\"
",
            container_name = container_name,
            step = escape_yaml_double(step_id),
            contract = escape_yaml_double(view.contract_id()),
        ));
    }
    if view.step_order().is_empty() {
        containers.push_str(&format!(
            "\
        - name: {name}-noop
          image: example.com/dpcs-step:scaffold
          command: [\"/bin/true\"]
"
        ));
    }
    containers
}

fn job_yaml(view: &PlanView<'_>, name: &str) -> String {
    format!(
        "\
# DPCS Kubernetes Job scaffold (experimental)
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
      restartPolicy: Never
      containers:
{containers}
",
        name = name,
        label = PlanView::k8s_name(view.contract_id()),
        containers = container_spec(view, name),
    )
}

fn cronjob_yaml(view: &PlanView<'_>, name: &str, cron: &str) -> String {
    format!(
        "\
# DPCS Kubernetes CronJob scaffold (experimental)
apiVersion: batch/v1
kind: CronJob
metadata:
  name: {name}
  labels:
    app.kubernetes.io/managed-by: dpcs
    dpcs.io/contract-id: \"{label}\"
spec:
  schedule: \"{cron}\"
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      template:
        metadata:
          labels:
            dpcs.io/contract-id: \"{label}\"
        spec:
          restartPolicy: Never
          containers:
{containers}
",
        name = name,
        label = PlanView::k8s_name(view.contract_id()),
        cron = escape_yaml_double(cron),
        containers = container_spec(view, name),
    )
}
