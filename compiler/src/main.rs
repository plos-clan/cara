const TEST_CODE: &str = r#"
const return_code = 0;

const main = extern C[main] fn() -> i32 {
    test_void();
    
    let mut return_code = return_code_getter(return_code);
    
    {
        return return_code;
    }
};

const test_void = fn() -> void {
    return;
};

const return_code_getter = fn(return_code: i32) -> i32 {
    let return_code = return_code;
    return_code
};
"#;

fn main() -> anyhow::Result<()> {
    codegen::init();

    let result = parser::parse(TEST_CODE)?;
    codegen::codegen(&result);

    Ok(())
}
