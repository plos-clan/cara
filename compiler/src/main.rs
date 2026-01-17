use std::{
    cell::LazyCell,
    process::{Command, exit},
    sync::Arc,
};

use analyzer::queries::CHECK_CONST_DEF;
use ast::{FileTable, ParseContext};
use codegen::{BackendOptions, CodegenBackendBase, EmitOptions, OutputType, codegen};
use codegen_llvm::LLVMBackend;
use parser::CaraParser;
use query::QueryContext;
use simplifier::simplify;
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
                crate_name,
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

            let mut file_table = FileTable::new();
            let file_id = file_table.register_file(input_file.clone())?;

            let ast = ParseContext::new(&file_table, file_id).parse(CaraParser::new())?;
            let ast = simplify(&mut file_table, crate_name.clone(), ast);

            let query_ctx = QueryContext::new(crate_name, Arc::new(ast));

            let main_fn = query_ctx.main_fn_id();
            let mut result = query_ctx.query(&CHECK_CONST_DEF, main_fn).unwrap();
            result.dump(query_ctx.clone(), &file_table);

            if result.has_error() {
                exit(-1);
            }

            let codegen_result = codegen(query_ctx, &LLVMBackend::new(backend_options));
            if release {
                codegen_result.optimize();
            }
            codegen_result.emit(emit_options);

            if matches!(emit, BuildResult::Executable) {
                let mut child = Command::new("gcc");
                child.arg("-o").arg(output_file).arg(temp_file.path());

                if matches!(reloc_mode, RelocMode::Static) {
                    child.arg("-static");
                }

                if !child.spawn()?.wait()?.success() {
                    anyhow::bail!("Failed to link executable");
                }
            }
        }
    }

    Ok(())
}
