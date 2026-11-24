//! Runs every module's test cases (`<module>/test/cases.json`) through the packages, the way the
//! Neblex test flow does: `item` is one test case, and the module's harness script returns a map
//! with `pass`. Each module runs with its own package, then with `DataQualityPackage`, which
//! catches two modules defining the same function differently.

use rhai::packages::Package;
use rhai::{Dynamic, Engine, Map, Scope};
use rhai_data_quality::{
    ContactsPackage, DataQualityPackage, DatesPackage, NumbersPackage, TextPackage,
};
use serde_json::Value;
use std::{fs, path::Path};

fn run_cases(module: &str, register: impl Fn(&mut Engine)) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(module).join("test");
    let mut engine = Engine::new();
    register(&mut engine);
    let harness = engine
        .compile(fs::read_to_string(dir.join("harness-script.rhai")).unwrap())
        .unwrap();

    let cases: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("cases.json")).unwrap()).unwrap();
    let cases = cases["cases"].as_array().expect("cases.json has a cases array");
    assert!(!cases.is_empty(), "{module}: no test cases");

    let failures: Vec<String> = cases
        .iter()
        .enumerate()
        .filter_map(|(i, case)| {
            let mut scope = Scope::new();
            scope.push("item", rhai::serde::to_dynamic(case).unwrap());
            let result = engine.eval_ast_with_scope::<Dynamic>(&mut scope, &harness);
            let map = result.as_ref().ok().and_then(|r| r.clone().try_cast::<Map>());
            let pass = map
                .as_ref()
                .and_then(|m| m.get("pass"))
                .and_then(|p| p.as_bool().ok())
                .unwrap_or(false);
            (!pass).then(|| {
                let got = match (&result, &map) {
                    (Err(e), _) => format!("error: {e}"),
                    (_, Some(m)) => m.get("got").map(|g| g.to_string()).unwrap_or_default(),
                    _ => String::new(),
                };
                format!("case {i}: {case} -> got {got}")
            })
        })
        .collect();

    assert!(
        failures.is_empty(),
        "{module}: {} of {} cases failed:\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
}

fn with<P: Package + Clone + 'static>(package: P) -> impl Fn(&mut Engine) {
    move |engine| {
        package.register_into_engine(engine);
    }
}

#[test]
fn contacts_alone() {
    run_cases("data-quality", with(ContactsPackage::new()));
}

#[test]
fn dates_alone() {
    run_cases("dates", with(DatesPackage::new()));
}

#[test]
fn numbers_alone() {
    run_cases("numbers", with(NumbersPackage::new()));
}

#[test]
fn text_alone() {
    run_cases("text", with(TextPackage::new()));
}

#[test]
fn all_modules_together() {
    let package = DataQualityPackage::new();
    for module in ["data-quality", "dates", "numbers", "text"] {
        run_cases(module, with(package.clone()));
    }
}

#[test]
fn namespaced_and_shared_between_engines() {
    let package = DatesPackage::new();
    let mut a = Engine::new();
    let mut b = Engine::new();
    package.register_into_engine_as(&mut a, "dates");
    package.register_into_engine(&mut b);

    let iso: String = a.eval(r#"dates::excel_serial_to_iso(45750)"#).unwrap();
    assert_eq!(iso, "2025-04-03");
    let week: String = b.eval(r#"iso_week("2027-01-01")"#).unwrap();
    assert_eq!(week, "2026-W53");
}
