const TEST_CODE: &str = r#"
const main = fn() -> i32 {
    test_void();
    {
        test()
    }
};

const test_void = fn() -> void {
    return;
};

const test = fn() -> i32 {
    42
};
"#;

fn main() -> anyhow::Result<()> {
    codegen::init();

    let result = parser::parse(TEST_CODE)?;
    codegen::codegen(&result);

    Ok(())
}
