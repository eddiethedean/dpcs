//! PyO3 bindings for `dpcs`.

use ::dpcs::{
    bind, compare_contracts, evaluate, parse_json, parse_target, parse_yaml, plan, to_json,
    to_yaml, validate, validate_conformance_profile, validate_registry, BindingResult,
    CapabilityProfile, CapabilityResult, CompatibilityResult, ConformanceProfile, PipelineContract,
    PlanResult, Registry, DPCS_SPEC_VERSION, VERSION,
};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use serde::Serialize;

fn to_py_err(err: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn report_dict(py: Python<'_>, value: impl Serialize) -> PyResult<PyObject> {
    let json = serde_json::to_string(&value).map_err(to_py_err)?;
    let obj: PyObject = py.import("json")?.call_method1("loads", (json,))?.into();
    Ok(obj)
}

#[pyfunction]
fn version() -> &'static str {
    VERSION
}

#[pyfunction]
fn dpcs_spec_version() -> &'static str {
    DPCS_SPEC_VERSION
}

#[pyfunction]
fn parse_yaml_str(py: Python<'_>, source: &str) -> PyResult<PyObject> {
    let contract = parse_yaml(source).map_err(|err| PyValueError::new_err(err.to_string()))?;
    report_dict(py, contract)
}

#[pyfunction]
fn parse_json_str(py: Python<'_>, source: &str) -> PyResult<PyObject> {
    let contract = parse_json(source).map_err(|err| PyValueError::new_err(err.to_string()))?;
    report_dict(py, contract)
}

#[pyfunction]
fn validate_yaml(py: Python<'_>, source: &str) -> PyResult<PyObject> {
    let contract = parse_yaml(source).map_err(|err| PyValueError::new_err(err.to_string()))?;
    report_dict(py, validate(&contract))
}

#[pyfunction]
fn validate_file(py: Python<'_>, path: &str) -> PyResult<PyObject> {
    let contract = PipelineContract::from_yaml_file(path)
        .or_else(|_| PipelineContract::from_json_file(path))
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    report_dict(py, validate(&contract))
}

#[pyfunction]
fn plan_yaml(py: Python<'_>, source: &str) -> PyResult<PyObject> {
    let contract = parse_yaml(source).map_err(|err| PyValueError::new_err(err.to_string()))?;
    match plan(&contract) {
        PlanResult::Ok(p) => report_dict(py, &*p),
        PlanResult::Err(report) => report_dict(py, report),
    }
}

#[pyfunction]
fn evaluate_capabilities(
    py: Python<'_>,
    profile_yaml: &str,
    contract_yaml: &str,
) -> PyResult<PyObject> {
    let profile = CapabilityProfile::from_yaml_str(profile_yaml)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    let contract =
        parse_yaml(contract_yaml).map_err(|err| PyValueError::new_err(err.to_string()))?;
    let planned = match plan(&contract) {
        PlanResult::Ok(p) => *p,
        PlanResult::Err(report) => return report_dict(py, report),
    };
    match evaluate(&planned, &profile) {
        CapabilityResult::Ok(report) => report_dict(py, &*report),
        CapabilityResult::Err { report, diagnostics } => {
            let mut payload = (*report).clone();
            payload.diagnostics = diagnostics.diagnostics;
            report_dict(py, payload)
        }
    }
}

#[pyfunction]
fn compare_contract_yaml(py: Python<'_>, baseline: &str, candidate: &str) -> PyResult<PyObject> {
    let baseline = parse_yaml(baseline).map_err(|err| PyValueError::new_err(err.to_string()))?;
    let candidate =
        parse_yaml(candidate).map_err(|err| PyValueError::new_err(err.to_string()))?;
    match compare_contracts(&baseline, &candidate) {
        CompatibilityResult::Ok(report) => report_dict(py, &*report),
        CompatibilityResult::Err { report, .. } => report_dict(py, &*report),
    }
}

#[pyfunction]
fn validate_registry_yaml(py: Python<'_>, source: &str) -> PyResult<PyObject> {
    let registry =
        Registry::from_yaml_str(source).map_err(|err| PyValueError::new_err(err.to_string()))?;
    report_dict(py, validate_registry(&registry))
}

#[pyfunction]
fn validate_conformance_profile_yaml(py: Python<'_>, source: &str) -> PyResult<PyObject> {
    let profile = ConformanceProfile::from_yaml_str(source)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    report_dict(py, validate_conformance_profile(&profile))
}

#[pyfunction]
fn bind_yaml(
    py: Python<'_>,
    contract_yaml: &str,
    profile_yaml: &str,
    target: &str,
) -> PyResult<PyObject> {
    let contract =
        parse_yaml(contract_yaml).map_err(|err| PyValueError::new_err(err.to_string()))?;
    let profile = CapabilityProfile::from_yaml_str(profile_yaml)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    let planned = match plan(&contract) {
        PlanResult::Ok(p) => *p,
        PlanResult::Err(report) => return report_dict(py, report),
    };
    let target = match parse_target(target) {
        Ok(t) => t,
        Err(report) => return report_dict(py, report),
    };
    match bind(&planned, &profile, target) {
        BindingResult::Ok(bundle) => report_dict(py, &*bundle),
        BindingResult::Err { diagnostics, .. } => report_dict(py, diagnostics),
    }
}

#[pyfunction]
fn to_yaml_str(contract_json: &str) -> PyResult<String> {
    let contract: PipelineContract =
        serde_json::from_str(contract_json).map_err(|err| PyValueError::new_err(err.to_string()))?;
    to_yaml(&contract).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pyfunction]
fn to_json_str(contract_yaml: &str) -> PyResult<String> {
    let contract =
        parse_yaml(contract_yaml).map_err(|err| PyValueError::new_err(err.to_string()))?;
    to_json(&contract).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pymodule]
fn dpcs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(dpcs_spec_version, m)?)?;
    m.add_function(wrap_pyfunction!(parse_yaml_str, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_str, m)?)?;
    m.add_function(wrap_pyfunction!(validate_yaml, m)?)?;
    m.add_function(wrap_pyfunction!(validate_file, m)?)?;
    m.add_function(wrap_pyfunction!(plan_yaml, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_capabilities, m)?)?;
    m.add_function(wrap_pyfunction!(compare_contract_yaml, m)?)?;
    m.add_function(wrap_pyfunction!(validate_registry_yaml, m)?)?;
    m.add_function(wrap_pyfunction!(validate_conformance_profile_yaml, m)?)?;
    m.add_function(wrap_pyfunction!(bind_yaml, m)?)?;
    m.add_function(wrap_pyfunction!(to_yaml_str, m)?)?;
    m.add_function(wrap_pyfunction!(to_json_str, m)?)?;
    m.add("__version__", VERSION)?;
    Ok(())
}
