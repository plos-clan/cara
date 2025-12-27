const TEST_CODE: &str = r#"
const main = fn() -> i32 {
    test();
    (*a)[3] + test()
};

const test = fn() -> i32 {
    42
};
"#;

fn main() -> anyhow::Result<()> {
    let result = parser::parse(TEST_CODE)?;
    println!("{:#?}", result);

    Ok(())
}
