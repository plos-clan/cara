const TEST_CODE: &str = r#"
const main = fn() -> i32 {
    (*a)[3] + 4
};
"#;

fn main() -> anyhow::Result<()> {
    let result = parser::parse(TEST_CODE)?;
    println!("{:#?}", result);

    Ok(())
}
