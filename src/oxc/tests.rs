use super::*;

fn js(src: &str) -> String {
    let alloc = allocator();
    let program = parse_js(&alloc, src).unwrap();
    Codegen::new().build(&program).code
}

fn ts_stripped(src: &str) -> String {
    let alloc = allocator();
    let program = parse_ts(&alloc, src).unwrap();
    strip_type_annotations(&alloc, program, "<test>").code
}

fn js_err(src: &str) -> String {
    let alloc = allocator();
    parse_js(&alloc, src).unwrap_err().to_string()
}

fn ts_err(src: &str) -> String {
    let alloc = allocator();
    parse_ts(&alloc, src).unwrap_err().to_string()
}

#[test]
fn parse_js_simple() {
    insta::assert_snapshot!(js("const x = 1 + 2;\n"));
}

#[test]
fn parse_js_function() {
    insta::assert_snapshot!(js("function add(a, b) { return a + b; }\n"));
}

#[test]
fn parse_js_syntax_error() {
    insta::assert_snapshot!(js_err("const x = {;"));
}

#[test]
fn parse_ts_with_annotations() {
    insta::assert_snapshot!(ts_stripped("const x: number = 42;\n"));
}

#[test]
fn parse_ts_interface() {
    insta::assert_snapshot!(ts_stripped(
        "interface Foo { bar: string; baz: number; }\nconst a = 1;\n"
    ));
}

#[test]
fn parse_ts_syntax_error() {
    insta::assert_snapshot!(ts_err("let x: = ;"));
}

#[test]
fn strip_types_from_function() {
    insta::assert_snapshot!(ts_stripped(
        "function add(a: number, b: number): number { return a + b; }\n"
    ));
}

#[test]
fn strip_preserves_real_exports() {
    insta::assert_snapshot!(ts_stripped(
        "export const x: number = 1;\nexport default function foo(): void {}\n"
    ));
}

#[test]
fn strip_removes_phantom_export() {
    // A type-only import gets stripped, leaving a phantom `export {};` that should be removed.
    insta::assert_snapshot!(ts_stripped(
        "import type { Foo } from './foo';\nconst x: Foo = { bar: 1 };\n"
    ));
}

#[test]
fn strip_type_alias() {
    insta::assert_snapshot!(ts_stripped(
        "type MyType = string | number;\nconst y = 'hello';\n"
    ));
}

#[test]
fn parse_js_rejects_ts_syntax() {
    insta::assert_snapshot!(js_err("const x: number = 1;\n"));
}
