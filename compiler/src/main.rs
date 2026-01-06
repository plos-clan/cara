use std::{fs::File, io::Read, path::Path};

use argh::FromArgs;
use codegen::codegen;
use query::QueryContext;

/// Cara compiler
#[derive(FromArgs)]
struct Args {
    #[argh(subcommand)]
    nested: Subcommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Build(BuildCommand),
}

/// Builds the cara file.
#[derive(FromArgs)]
#[argh(subcommand, name = "build")]
struct BuildCommand {
    #[argh(positional)]
    input_file: String,
    /// the output path.
    #[argh(option, default = "String::from(\"a.out\")", short = 'o')]
    output_file: String,
}

fn main() -> anyhow::Result<()> {
    codegen::init();

    let args = argh::from_env::<Args>();

    match args.nested {
        Subcommand::Build(build) => {
            let BuildCommand {
                input_file,
                output_file,
            } = build;

            let mut source_code = String::new();
            File::open(input_file)?.read_to_string(&mut source_code)?;

            let ast = parser::parse(&source_code)?;
            let query_ctx = QueryContext::new(&ast);

            let codegen_result = codegen(query_ctx);
            codegen_result.dump();
            codegen_result.optimize();
            codegen_result.dump();
            codegen_result.write_to_file(Path::new(&output_file));
        }
    }

    Ok(())
}
