use codegen::codegen;
use query::QueryContext;

const TEST_CODE: &str = include_str!("main.cara");

fn main() -> anyhow::Result<()> {
    codegen::init();

    let result = parser::parse(TEST_CODE)?;
    let query_ctx = QueryContext::new(&result);
    codegen(query_ctx);

    Ok(())
}
