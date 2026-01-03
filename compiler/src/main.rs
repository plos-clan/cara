const TEST_CODE: &str = r#"
const return_code = 0;

const main = extern C[main] fn() -> i64 {
    fib(10i64)
};

const fib = fn(n: i64) -> i64 {
    if n <= 1i64 {
        n
    } else {
        fib(n - 1i64) + fib(n - 2i64)
    }
};

"#;

fn main() -> anyhow::Result<()> {
    codegen::init();

    let result = parser::parse(TEST_CODE)?;
    codegen::codegen(&result);

    Ok(())
}
