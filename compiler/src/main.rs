use std::{cell::LazyCell, fs::File, io::Read, process::Command};

use codegen::{BackendOptions, CodegenBackendBase, EmitOptions, OutputType, codegen};
use codegen_llvm::LLVMBackend;
use query::QueryContext;
use tempfile::NamedTempFile;

use args::*;

mod args;

fn main() -> anyhow::Result<()> {
    let args = argh::from_env::<Args>();

    match args.nested {
        Subcommand::Build(build) => {
            let BuildCommand {
                input_file,
                output_file,
                emit,
                code_model,
                optimize_level,
                reloc_mode,
                release,
            } = build;

            let temp_file = LazyCell::new(|| NamedTempFile::new().unwrap());

            let output_path = match emit {
                BuildResult::Executable => temp_file.path().to_str().unwrap().into(),
                _ => output_file.clone(),
            };

            let backend_options = BackendOptions::builder()
                .code_model(code_model.into())
                .optimize_level(optimize_level.into())
                .reloc_mode(reloc_mode.into())
                .build();

            let emit_options = EmitOptions::builder()
                .path(output_path)
                .output_type(match emit {
                    BuildResult::Ir => OutputType::Ir,
                    BuildResult::Asm => OutputType::Asm,
                    BuildResult::Object | BuildResult::Executable => OutputType::Object,
                })
                .build();

            let mut source_code = String::new();
            File::open(input_file)?.read_to_string(&mut source_code)?;

            let ast = parser::parse(&source_code)?;
            let query_ctx = QueryContext::new(&ast);

            let codegen_result = codegen(query_ctx, &LLVMBackend::new(backend_options));
            if release {
                codegen_result.optimize();
            }
            codegen_result.emit(emit_options);

            if matches!(emit, BuildResult::Executable) {
                let mut child = Command::new("gcc")
                    .arg("-o")
                    .arg(output_file)
                    .arg(temp_file.path())
                    .spawn()?;

                child.wait()?;
            }
        }
    }

    Ok(())
}
