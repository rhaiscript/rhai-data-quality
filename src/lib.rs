//! Rhai packages for cleaning business data: contact and company records, dates, numbers and
//! money, and text.
//!
//! Every function is written in plain Rhai, is pure, and uses only core Rhai, so it behaves the
//! same in any engine. The packages compile the bundled scripts once; register a package with as
//! many engines as you like.
//!
//! # Quick start
//!
//! ```
//! use rhai::Engine;
//! use rhai::packages::Package;
//! use rhai_data_quality::DataQualityPackage;
//!
//! let mut engine = Engine::new();
//! DataQualityPackage::new().register_into_engine(&mut engine);
//!
//! let email: String = engine.eval(r#"normalize_email(" Jane.Doe@Example.COM ")"#).unwrap();
//! assert_eq!(email, "jane.doe@example.com");
//!
//! let date: String = engine.eval(r#"to_iso_date("April 3rd, 2026", "mdy")"#).unwrap();
//! assert_eq!(date, "2026-04-03");
//! ```
//!
//! # One module, under a namespace
//!
//! ```
//! use rhai::Engine;
//! use rhai::packages::Package;
//! use rhai_data_quality::{ContactsPackage, NumbersPackage};
//!
//! let mut engine = Engine::new();
//! ContactsPackage::new().register_into_engine_as(&mut engine, "contacts");
//! NumbersPackage::new().register_into_engine_as(&mut engine, "num");
//!
//! let phone: String = engine.eval(r#"contacts::phone_e164("(415) 555-0123", "1")"#).unwrap();
//! assert_eq!(phone, "+14155550123");
//!
//! let cents: i64 = engine.eval(r#"num::to_cents("1.234,56 €", "auto")"#).unwrap();
//! assert_eq!(cents, 123456);
//! ```
//!
//! # Packages
//!
//! | Package | Script | What it does |
//! |---|---|---|
//! | [`DataQualityPackage`] | all four | everything below |
//! | [`ContactsPackage`] | `data-quality.rhai` | emails, phone numbers, names, company names, domains, URLs, postcodes, states, countries, Luhn check digits, match keys for finding duplicates, missing fields |
//! | [`DatesPackage`] | `dates.rhai` | dates in mixed formats to `YYYY-MM-DD`, Excel serial dates, Unix timestamps, business days, ISO weeks, fiscal periods, months and ages |
//! | [`NumbersPackage`] | `numbers.rhai` | amounts in US or European formats, percentages, currency codes, banker's rounding, display formatting, durations, amounts in words |
//! | [`TextPackage`] | `text.rhai` | accents, slugs, truncation, masking, padding, HTML snippets, title, snake, camel and kebab case |
//!
//! The full function list, with examples and limits, is in the
//! [README](https://github.com/rhaiscript/rhai-data-quality#readme).
//!
//! # Requirements
//!
//! The scripts use floating-point numbers, arrays, object maps and functions, so the engine must
//! not use Rhai's `no_float`, `no_index`, `no_object`, `no_function` or `no_module` features. If
//! your engine uses `unchecked`, turn on this crate's `unchecked` feature as well.
//!
//! The script sources are also available in [`sources`], to load them another way (for example,
//! as files for a module resolver).

#![warn(missing_docs)]

use rhai::packages::Package;
use rhai::{Engine, Module, Scope, Shared, AST};

/// The bundled Rhai scripts, as source text.
pub mod sources {
    /// `data-quality.rhai`: contact and company records.
    pub const DATA_QUALITY: &str = include_str!("../data-quality/data-quality.rhai");
    /// `dates.rhai`: dates.
    pub const DATES: &str = include_str!("../dates/dates.rhai");
    /// `numbers.rhai`: numbers and money.
    pub const NUMBERS: &str = include_str!("../numbers/numbers.rhai");
    /// `text.rhai`: text.
    pub const TEXT: &str = include_str!("../text/text.rhai");
    /// Every script, in load order.
    pub const ALL: [&str; 4] = [DATA_QUALITY, DATES, NUMBERS, TEXT];
}

/// Compiles the scripts into one module whose functions can call each other.
fn build_module(scripts: &[&str]) -> Module {
    let mut engine = Engine::new();
    // Debug builds of Rhai default to half the nesting depth of release builds, which is too
    // shallow for some of the scripts. These are the release defaults.
    #[cfg(not(feature = "unchecked"))]
    engine.set_max_expr_depths(64, 32);

    let ast = scripts.iter().fold(AST::empty(), |ast, script| {
        ast.merge(&engine.compile(script).expect("bundled script should compile"))
    });
    Module::eval_ast_as_new(Scope::new(), &ast, &engine).expect("bundled script should load")
}

macro_rules! script_package {
    ($(#[$attr:meta])* $name:ident, $id:literal, [$($script:expr),+]) => {
        $(#[$attr])*
        #[derive(Debug, Clone)]
        pub struct $name(Shared<Module>);

        impl $name {
            /// Compiles the bundled scripts into a new package.
            ///
            /// Compiling takes a few milliseconds; create the package once and clone it, or
            /// register it with each engine, rather than creating it per engine.
            #[must_use]
            pub fn new() -> Self {
                let mut module = Module::new();
                <Self as Package>::init(&mut module);
                module.set_id($id);
                module.build_index();
                Self(module.into())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Package for $name {
            fn init(module: &mut Module) {
                module.combine(build_module(&[$($script),+]));
            }

            fn as_shared_module(&self) -> Shared<Module> {
                self.0.clone()
            }
        }
    };
}

script_package!(
    /// Every function in this crate: [`ContactsPackage`], [`DatesPackage`], [`NumbersPackage`]
    /// and [`TextPackage`] together.
    DataQualityPackage,
    "rhai_data_quality",
    [sources::DATA_QUALITY, sources::DATES, sources::NUMBERS, sources::TEXT]
);

script_package!(
    /// Contact and company records (`data-quality.rhai`): emails, phone numbers, names, company
    /// names, domains, URLs, US ZIP codes and states, Canadian and UK postcodes, country codes,
    /// Luhn check digits, match keys for finding duplicates, and missing fields.
    ContactsPackage,
    "rhai_data_quality::contacts",
    [sources::DATA_QUALITY]
);

script_package!(
    /// Dates (`dates.rhai`): dates written in common formats to `YYYY-MM-DD`, Excel serial dates,
    /// Unix timestamps, days and business days between and added, weekdays, ISO weeks, month
    /// ends, quarters, fiscal years and quarters, months and ages.
    DatesPackage,
    "rhai_data_quality::dates",
    [sources::DATES]
);

script_package!(
    /// Numbers and money (`numbers.rhai`): numbers and amounts in US or European formats,
    /// percentages, currency codes, banker's rounding, display formatting, percent change, safe
    /// division, durations, currency rounding, amounts in words and clamping.
    NumbersPackage,
    "rhai_data_quality::numbers",
    [sources::NUMBERS]
);

script_package!(
    /// Text (`text.rhai`): accents, slugs, truncation at a word, initials, masking, left padding,
    /// HTML snippets, word counts, title, snake, camel and kebab case, hashtags and mentions, and
    /// typographic quotes and dashes.
    TextPackage,
    "rhai_data_quality::text",
    [sources::TEXT]
);
