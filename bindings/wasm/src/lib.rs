//! WASM bindings for `dpcs`.

use dpcs::{
    bind, compare_contracts, evaluate, parse_json, parse_yaml, plan, to_json, validate,
    validate_registry, CapabilityProfile, Registry, DPCS_SPEC_VERSION, VERSION,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|err| JsValue::from_str(&err.to_string()))
}

fn err_js(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn version() -> String {
    VERSION.to_owned()
}

#[wasm_bindgen]
pub fn dpcs_spec_version() -> String {
    DPCS_SPEC_VERSION.to_owned()
}

#[wasm_bindgen]
pub fn validate_yaml(source: &str) -> Result<JsValue, JsValue> {
    let contract = parse_yaml(source).map_err(err_js)?;
    to_js(&validate(&contract))
}

#[wasm_bindgen]
pub fn validate_json(source: &str) -> Result<JsValue, JsValue> {
    let contract = parse_json(source).map_err(err_js)?;
    to_js(&validate(&contract))
}

#[wasm_bindgen]
pub fn plan_yaml(source: &str) -> Result<JsValue, JsValue> {
    let contract = parse_yaml(source).map_err(err_js)?;
    match plan(&contract) {
        dpcs::PlanResult::Ok(p) => to_js(&*p),
        dpcs::PlanResult::Err(report) => to_js(&report),
    }
}

#[wasm_bindgen]
pub fn evaluate_capabilities(profile_yaml: &str, contract_yaml: &str) -> Result<JsValue, JsValue> {
    let profile = CapabilityProfile::from_yaml_str(profile_yaml).map_err(err_js)?;
    let contract = parse_yaml(contract_yaml).map_err(err_js)?;
    let planned = match plan(&contract) {
        dpcs::PlanResult::Ok(p) => *p,
        dpcs::PlanResult::Err(report) => return to_js(&report),
    };
    match evaluate(&planned, &profile) {
        dpcs::CapabilityResult::Ok(report) => to_js(&*report),
        dpcs::CapabilityResult::Err {
            report,
            diagnostics,
        } => {
            let mut payload = (*report).clone();
            payload.diagnostics = diagnostics.diagnostics;
            to_js(&payload)
        }
    }
}

#[wasm_bindgen]
pub fn compare_contract_yaml(baseline: &str, candidate: &str) -> Result<JsValue, JsValue> {
    let baseline = parse_yaml(baseline).map_err(err_js)?;
    let candidate = parse_yaml(candidate).map_err(err_js)?;
    match compare_contracts(&baseline, &candidate) {
        dpcs::CompatibilityResult::Ok(report) => to_js(&*report),
        dpcs::CompatibilityResult::Err { report, .. } => to_js(&*report),
    }
}

#[wasm_bindgen]
pub fn validate_registry_yaml(source: &str) -> Result<JsValue, JsValue> {
    let registry = Registry::from_yaml_str(source).map_err(err_js)?;
    to_js(&validate_registry(&registry))
}

#[wasm_bindgen]
pub fn bind_yaml(
    contract_yaml: &str,
    profile_yaml: &str,
    target: &str,
) -> Result<JsValue, JsValue> {
    let contract = parse_yaml(contract_yaml).map_err(err_js)?;
    let profile = CapabilityProfile::from_yaml_str(profile_yaml).map_err(err_js)?;
    let planned = match plan(&contract) {
        dpcs::PlanResult::Ok(p) => *p,
        dpcs::PlanResult::Err(report) => return to_js(&report),
    };
    let target = match dpcs::parse_target(target) {
        Ok(t) => t,
        Err(report) => return to_js(&report),
    };
    match bind(&planned, &profile, target) {
        dpcs::BindingResult::Ok(bundle) => to_js(&*bundle),
        dpcs::BindingResult::Err { diagnostics, .. } => to_js(&diagnostics),
    }
}

#[wasm_bindgen]
pub fn contract_to_json(source_yaml: &str) -> Result<String, JsValue> {
    let contract = parse_yaml(source_yaml).map_err(err_js)?;
    to_json(&contract).map_err(err_js)
}
