use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};
use crate::ast_util::is_function_node;

fn max_statements_diagnostic(name: &str, count: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "{name} has too many statements ({count}). Maximum allowed is {max}."
    ))
    .with_help("Consider splitting it into smaller functions.")
    .with_label(span)
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MaxStatementsConfig {
    /// Maximum number of statements allowed per function.
    max: usize,
    /// Whether to ignore top-level functions.
    ignore_top_level_functions: bool,
}

const DEFAULT_MAX_LINES_PER_FUNCTION: usize = 10;

impl Default for MaxStatementsConfig {
    fn default() -> Self {
        Self { max: DEFAULT_MAX_LINES_PER_FUNCTION, ignore_top_level_functions: false }
    }
}

// TODO: check Deserialize and Serialize
#[derive(Debug, Default, Clone)]
pub struct MaxStatements(MaxStatementsConfig);

impl std::ops::Deref for MaxStatements {
    type Target = MaxStatementsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    MaxStatements,
    eslint,
    pedantic,
    config = MaxStatementsConfig,
);

impl Rule for MaxStatements {
    fn from_configuration(value: Value) -> Self {
        let config = value.get(0);
        let config = if let Some(max) = config
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            MaxStatementsConfig {
                max,
                ignore_top_level_functions: false,
            }
        } else {
            let max = config
                .and_then(|config| config.get("max"))
                .and_then(Value::as_number)
                .and_then(serde_json::Number::as_u64)
                .map_or(DEFAULT_MAX_LINES_PER_FUNCTION, |v| {
                    usize::try_from(v).unwrap_or(DEFAULT_MAX_LINES_PER_FUNCTION)
                });
            let ignore_top_level_functions = config
                .and_then(|config| config.get("ignoreTopLevelFunctions"))
                .and_then(Value::as_bool)
                .unwrap_or(false);

            MaxStatementsConfig {max, ignore_top_level_functions}
        };

        Self(config)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        print!("{}", "TEST IS RUNNING");
        if !is_function_node(node) {
            return;
        }
        let source = ctx.source_text();
        print!("{}", source);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "function foo() { var bar = 1; function qux () { var noCount = 2; } return 3; }",
            Some(serde_json::json!([3])),
        ),
        (
            "function foo() { var bar = 1; if (true) { for (;;) { var qux = null; } } else { quxx(); } return 3; }",
            Some(serde_json::json!([6])),
        ),
        (
            "function foo() { var x = 5; function bar() { var y = 6; } bar(); z = 10; baz(); }",
            Some(serde_json::json!([5])),
        ),
        (
            "function foo() { var a; var b; var c; var x; var y; var z; bar(); baz(); qux(); quxx(); }",
            None,
        ),
        (
            "(function() { var bar = 1; return function () { return 42; }; })()",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "function foo() { var bar = 1; var baz = 2; }",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "define(['foo', 'qux'], function(foo, qux) { var bar = 1; var baz = 2; })",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "var foo = { thing: function() { var bar = 1; var baz = 2; } }",
            Some(serde_json::json!([2])),
        ),
        ("var foo = { thing() { var bar = 1; var baz = 2; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 6 },
        ("var foo = { ['thing']() { var bar = 1; var baz = 2; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 6 },
        ("var foo = { thing: () => { var bar = 1; var baz = 2; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 6 },
        (
            "var foo = { thing: function() { var bar = 1; var baz = 2; } }",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "class C { static { one; two; three; { four; five; six; } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { class C { static { one; two; three; { four; five; six; } } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { one; two; three; function foo() { 1; 2; } four; five; six; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { { one; two; three; function foo() { 1; 2; } four; five; six; } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function top_level() { 1; /* 2 */ class C { static { one; two; three; { four; five; six; } } } 3;}",
            Some(serde_json::json!([2, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "function top_level() { 1; 2; } class C { static { one; two; three; { four; five; six; } } }",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { one; two; three; { four; five; six; } } } function top_level() { 1; 2; } ",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { let one; let two = class { static { let three; let four; let five; if (six) { let seven; let eight; let nine; } } }; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        ("function foo() { var bar = 1; var baz = 2; var qux = 3; }", Some(serde_json::json!([2]))),
        (
            "var foo = () => { var bar = 1; var baz = 2; var qux = 3; };",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function() { var bar = 1; var baz = 2; var qux = 3; };",
            Some(serde_json::json!([2])),
        ),
        (
            "function foo() { var bar = 1; if (true) { while (false) { var qux = null; } } return 3; }",
            Some(serde_json::json!([4])),
        ),
        (
            "function foo() { var bar = 1; if (true) { for (;;) { var qux = null; } } return 3; }",
            Some(serde_json::json!([4])),
        ),
        (
            "function foo() { var bar = 1; if (true) { for (;;) { var qux = null; } } else { quxx(); } return 3; }",
            Some(serde_json::json!([5])),
        ),
        (
            "function foo() { var x = 5; function bar() { var y = 6; } bar(); z = 10; baz(); }",
            Some(serde_json::json!([3])),
        ),
        (
            "function foo() { var x = 5; function bar() { var y = 6; } bar(); z = 10; baz(); }",
            Some(serde_json::json!([4])),
        ),
        (
            ";(function() { var bar = 1; return function () { var z; return 42; }; })()",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            ";(function() { var bar = 1; var baz = 2; })(); (function() { var bar = 1; var baz = 2; })()",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "define(['foo', 'qux'], function(foo, qux) { var bar = 1; var baz = 2; return function () { var z; return 42; }; })",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "function foo() { var a; var b; var c; var x; var y; var z; bar(); baz(); qux(); quxx(); foo(); }",
            None,
        ),
        (
            "var foo = { thing: function() { var bar = 1; var baz = 2; var baz2; } }",
            Some(serde_json::json!([2])),
        ),
        (
            "var foo = { thing() { var bar = 1; var baz = 2; var baz2; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = { thing: () => { var bar = 1; var baz = 2; var baz2; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = { thing: function() { var bar = 1; var baz = 2; var baz2; } }",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        ("function foo() { 1; 2; 3; 4; 5; 6; 7; 8; 9; 10; 11; }", Some(serde_json::json!([{}]))),
        ("function foo() { 1; }", Some(serde_json::json!([{ "max": 0 }]))),
        (
            "function foo() { foo_1; /* foo_ 2 */ class C { static { one; two; three; four; { five; six; seven; eight; } } } foo_3 }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { one; two; three; four; function not_top_level() { 1; 2; 3; } five; six; seven; eight; } }",
            Some(serde_json::json!([2, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { { one; two; three; four; function not_top_level() { 1; 2; 3; } five; six; seven; eight; } } }",
            Some(serde_json::json!([2, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { { one; two; three; four; } function not_top_level() { 1; 2; 3; } { five; six; seven; eight; } } }",
            Some(serde_json::json!([2, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 }
    ];

    Tester::new(MaxStatements::NAME, MaxStatements::PLUGIN, pass, fail).test_and_snapshot();
}
