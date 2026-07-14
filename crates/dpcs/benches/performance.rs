//! Criterion benchmarks for ROADMAP 0.12 performance work.

use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dpcs::{plan, synth, to_json, to_yaml, validate, validate_sequential, DependencyGraph};

fn example_yaml() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/minimal.dpcs.yaml");
    std::fs::read_to_string(path).expect("read minimal example")
}

fn configure(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_secs(2));
}

fn bench_parse(c: &mut Criterion) {
    let yaml = example_yaml();
    let json = {
        let contract = dpcs::parse_yaml(&yaml).expect("parse");
        to_json(&contract).expect("json")
    };

    let mut group = c.benchmark_group("parse");
    configure(&mut group);
    group.bench_function("yaml_minimal", |b| {
        b.iter(|| dpcs::parse_yaml(black_box(&yaml)).expect("parse"))
    });
    group.bench_function("json_minimal", |b| {
        b.iter(|| dpcs::parse_json(black_box(&json)).expect("parse"))
    });
    group.finish();
}

fn bench_validate(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate");
    configure(&mut group);
    for n in [100usize, 500, 1000] {
        let contract = synth::linear_pipeline(n);
        group.bench_with_input(
            BenchmarkId::new("sequential_linear", n),
            &contract,
            |b, c| b.iter(|| validate_sequential(black_box(c))),
        );
        group.bench_with_input(BenchmarkId::new("validate_linear", n), &contract, |b, c| {
            b.iter(|| validate(black_box(c)))
        });
    }
    let dense = synth::dense_pipeline(500, 3);
    group.bench_function("validate_dense_500", |b| {
        b.iter(|| validate(black_box(&dense)))
    });
    group.finish();
}

fn bench_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph");
    configure(&mut group);
    for n in [100usize, 500, 1000] {
        let contract = synth::linear_pipeline(n);
        group.bench_with_input(BenchmarkId::new("from_contract", n), &contract, |b, c| {
            b.iter(|| DependencyGraph::from_contract(black_box(c)))
        });
        let graph = DependencyGraph::from_contract(&contract);
        group.bench_with_input(BenchmarkId::new("topo", n), &graph, |b, g| {
            b.iter(|| g.topological_order().expect("topo"))
        });
        group.bench_with_input(
            BenchmarkId::new("unreachable", n),
            &(graph, contract),
            |b, (g, c)| b.iter(|| g.unreachable_steps(black_box(c))),
        );
    }
    group.finish();
}

fn bench_plan_and_serialize(c: &mut Criterion) {
    let contract = synth::linear_pipeline(200);
    let mut group = c.benchmark_group("plan_serialize");
    configure(&mut group);
    group.bench_function("plan_200", |b| b.iter(|| plan(black_box(&contract))));
    group.bench_function("to_yaml_200", |b| {
        b.iter(|| to_yaml(black_box(&contract)).expect("yaml"))
    });
    group.bench_function("to_json_200", |b| {
        b.iter(|| to_json(black_box(&contract)).expect("json"))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_parse,
    bench_validate,
    bench_graph,
    bench_plan_and_serialize
);
criterion_main!(benches);
