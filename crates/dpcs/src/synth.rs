//! Synthetic Pipeline Contracts for benchmarks and scale tests.
//!
//! These helpers build owned COM trees in memory (no YAML round-trip) so
//! large-graph suites stay fast and deterministic.

use crate::model::{
    DataFlow, GraphEdge, InterfacePort, PipelineContract, PipelineGraph, PipelineInterface,
    PipelineStep, StepPort,
};

/// Build a linear pipeline with `n` steps connected by graph edges `s0→s1→…→s{n-1}`.
///
/// Includes interface-rooted data flow so the contract validates cleanly for
/// large-N benches.
pub fn linear_pipeline(n: usize) -> PipelineContract {
    let n = n.max(1);
    let mut steps = Vec::with_capacity(n);
    let mut edges = Vec::with_capacity(n.saturating_sub(1));
    let mut data_flow = Vec::new();

    for i in 0..n {
        let id = format!("s{i}");
        steps.push(step_with_ports(&id));
        if i + 1 < n {
            let next = format!("s{}", i + 1);
            edges.push(edge(&id, &next));
            data_flow.push(DataFlow {
                from: format!("steps.{id}.outputs.out"),
                to: format!("steps.{next}.inputs.in"),
                dataset: Some(format!("ds{i}")),
                contract_ref: None,
                extensions: Default::default(),
            });
        }
    }

    // Interface roots for dataset reachability / port satisfaction.
    data_flow.insert(
        0,
        DataFlow {
            from: "interface.inputs.raw".into(),
            to: "steps.s0.inputs.in".into(),
            dataset: Some("raw".into()),
            contract_ref: None,
            extensions: Default::default(),
        },
    );
    let last = format!("s{}", n - 1);
    data_flow.push(DataFlow {
        from: format!("steps.{last}.outputs.out"),
        to: "interface.outputs.clean".into(),
        dataset: Some("clean".into()),
        contract_ref: None,
        extensions: Default::default(),
    });

    base_contract(
        format!("synth.linear.{n}"),
        steps,
        edges,
        data_flow,
        vec!["s0".into()],
        true,
    )
}

/// Build a fan-out / fan-in DAG with `width` parallel mid steps under one root and sink.
pub fn dag_pipeline(width: usize) -> PipelineContract {
    let mid = width.max(1);
    let mut steps = vec![step_with_ports("root"), step_with_ports("sink")];
    let mut edges = Vec::with_capacity(mid * 2);
    let mut data_flow = vec![DataFlow {
        from: "interface.inputs.raw".into(),
        to: "steps.root.inputs.in".into(),
        dataset: Some("raw".into()),
        contract_ref: None,
        extensions: Default::default(),
    }];

    for i in 0..mid {
        let id = format!("m{i}");
        steps.push(step_with_ports(&id));
        edges.push(edge("root", &id));
        edges.push(edge(&id, "sink"));
        data_flow.push(DataFlow {
            from: "steps.root.outputs.out".into(),
            to: format!("steps.{id}.inputs.in"),
            dataset: Some(format!("to-{id}")),
            contract_ref: None,
            extensions: Default::default(),
        });
        data_flow.push(DataFlow {
            from: format!("steps.{id}.outputs.out"),
            to: "steps.sink.inputs.in".into(),
            dataset: Some(format!("from-{id}")),
            contract_ref: None,
            extensions: Default::default(),
        });
    }
    data_flow.push(DataFlow {
        from: "steps.sink.outputs.out".into(),
        to: "interface.outputs.clean".into(),
        dataset: Some("clean".into()),
        contract_ref: None,
        extensions: Default::default(),
    });

    base_contract(
        format!("synth.dag.{mid}"),
        steps,
        edges,
        data_flow,
        vec!["root".into()],
        true,
    )
}

/// Build a denser graph: each step links to the next `span` successors (clipped).
pub fn dense_pipeline(n: usize, span: usize) -> PipelineContract {
    let span = span.max(1);
    let n = n.max(1);
    let mut steps = Vec::with_capacity(n);
    let mut edges = Vec::new();

    for i in 0..n {
        steps.push(step_bare(&format!("s{i}")));
        for offset in 1..=span {
            let j = i + offset;
            if j < n {
                edges.push(edge(&format!("s{i}"), &format!("s{j}")));
            }
        }
    }

    base_contract(
        format!("synth.dense.{n}.{span}"),
        steps,
        edges,
        Vec::new(),
        vec!["s0".into()],
        false,
    )
}

/// Build `n` steps with wide interface-rooted data-flow fan-in/out (no graph edges).
pub fn wide_data_flow(n: usize) -> PipelineContract {
    let mut steps = Vec::with_capacity(n);
    let mut inputs = Vec::with_capacity(n);
    let mut outputs = Vec::with_capacity(n);
    let mut data_flow = Vec::with_capacity(n * 2);

    for i in 0..n {
        let id = format!("s{i}");
        let in_port = format!("in{i}");
        let out_port = format!("out{i}");
        steps.push(PipelineStep {
            id: id.clone(),
            step_type: "extension:noop".into(),
            name: None,
            contract_ref: None,
            transform_ref: None,
            inputs: vec![StepPort {
                id: in_port.clone(),
                contract_ref: None,
                extensions: Default::default(),
            }],
            outputs: vec![StepPort {
                id: out_port.clone(),
                contract_ref: None,
                extensions: Default::default(),
            }],
            extensions: Default::default(),
        });
        inputs.push(InterfacePort {
            id: in_port.clone(),
            name: Some(format!("Input {i}")),
            contract_ref: Some(format!("contracts/in{i}.odcs.yaml")),
            purpose: Some(format!("Synthetic input {i}")),
            extensions: Default::default(),
        });
        outputs.push(InterfacePort {
            id: out_port.clone(),
            name: Some(format!("Output {i}")),
            contract_ref: Some(format!("contracts/out{i}.odcs.yaml")),
            purpose: Some(format!("Synthetic output {i}")),
            extensions: Default::default(),
        });
        data_flow.push(DataFlow {
            from: format!("interface.inputs.{in_port}"),
            to: format!("steps.{id}.inputs.{in_port}"),
            dataset: Some(format!("raw{i}")),
            contract_ref: None,
            extensions: Default::default(),
        });
        data_flow.push(DataFlow {
            from: format!("steps.{id}.outputs.{out_port}"),
            to: format!("interface.outputs.{out_port}"),
            dataset: Some(format!("clean{i}")),
            contract_ref: None,
            extensions: Default::default(),
        });
    }

    PipelineContract {
        dpcs_version: "1.0.0".into(),
        id: format!("synth.wide.{n}"),
        name: Some("Synthetic wide data-flow".into()),
        version: "0.1.0".into(),
        metadata: None,
        interface: PipelineInterface {
            metadata: None,
            inputs,
            outputs,
            extensions: Default::default(),
        },
        graph: PipelineGraph {
            entry_points: steps
                .first()
                .map(|s| vec![s.id.clone()])
                .unwrap_or_default(),
            exit_points: Vec::new(),
            metadata: None,
            edges: Vec::new(),
            extensions: Default::default(),
        },
        steps,
        contract_references: Vec::new(),
        data_flow,
        control_flow: Vec::new(),
        execution: None,
        scheduling: Vec::new(),
        quality_gates: Vec::new(),
        failure_semantics: Vec::new(),
        lineage: None,
        compatibility: None,
        security: None,
        governance: None,
        extensions: Default::default(),
    }
}

fn step_with_ports(id: &str) -> PipelineStep {
    PipelineStep {
        id: id.into(),
        step_type: "extension:noop".into(),
        name: None,
        contract_ref: None,
        transform_ref: None,
        inputs: vec![StepPort {
            id: "in".into(),
            contract_ref: None,
            extensions: Default::default(),
        }],
        outputs: vec![StepPort {
            id: "out".into(),
            contract_ref: None,
            extensions: Default::default(),
        }],
        extensions: Default::default(),
    }
}

fn step_bare(id: &str) -> PipelineStep {
    PipelineStep {
        id: id.into(),
        step_type: "extension:noop".into(),
        name: None,
        contract_ref: None,
        transform_ref: None,
        inputs: Vec::new(),
        outputs: Vec::new(),
        extensions: Default::default(),
    }
}

fn edge(from: &str, to: &str) -> GraphEdge {
    GraphEdge {
        from: from.into(),
        to: to.into(),
        kind: None,
        extensions: Default::default(),
    }
}

fn base_contract(
    id: String,
    steps: Vec<PipelineStep>,
    edges: Vec<GraphEdge>,
    data_flow: Vec<DataFlow>,
    entry_points: Vec<String>,
    with_interface: bool,
) -> PipelineContract {
    let interface = if with_interface {
        PipelineInterface {
            metadata: None,
            inputs: vec![InterfacePort {
                id: "raw".into(),
                name: Some("Raw".into()),
                contract_ref: Some("contracts/raw.odcs.yaml".into()),
                purpose: Some("Synthetic raw input".into()),
                extensions: Default::default(),
            }],
            outputs: vec![InterfacePort {
                id: "clean".into(),
                name: Some("Clean".into()),
                contract_ref: Some("contracts/clean.odcs.yaml".into()),
                purpose: Some("Synthetic clean output".into()),
                extensions: Default::default(),
            }],
            extensions: Default::default(),
        }
    } else {
        PipelineInterface {
            metadata: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            extensions: Default::default(),
        }
    };

    PipelineContract {
        dpcs_version: "1.0.0".into(),
        id,
        name: Some("Synthetic pipeline".into()),
        version: "0.1.0".into(),
        metadata: None,
        interface,
        graph: PipelineGraph {
            entry_points,
            exit_points: Vec::new(),
            metadata: None,
            edges,
            extensions: Default::default(),
        },
        steps,
        contract_references: Vec::new(),
        data_flow,
        control_flow: Vec::new(),
        execution: None,
        scheduling: Vec::new(),
        quality_gates: Vec::new(),
        failure_semantics: Vec::new(),
        lineage: None,
        compatibility: None,
        security: None,
        governance: None,
        extensions: Default::default(),
    }
}
