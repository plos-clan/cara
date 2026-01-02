const TEST_CODE: &str = r#"
const return_code = -(114 + 5);

const main = extern C[main] fn() -> i32 {
    test_void();
    {
        return -return_code_getter();
    }
};

const test_void = fn() -> void {
    return;
};

const return_code_getter = fn() -> i32 {
    return_code
};
"#;

fn main() -> anyhow::Result<()> {
    codegen::init();

    let result = parser::parse(TEST_CODE)?;
    codegen::codegen(&result);

    Ok(())
}
