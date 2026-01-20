use std::{cell::LazyCell, path::Path, process::exit, sync::Arc};

use analyzer::queries::CHECK_CONST_DEF;
use anyhow::bail;
use ast::{FileTable, ParseContext};
use clap::Parser;
use codegen::{BackendOptions, CodegenBackendBase, EmitOptions, OutputType, codegen};
use codegen_llvm::LLVMBackend;
use parser::CaraParser;
use query::QueryContext;
use simplifier::simplify;
use targets::{
    linker::{Cc, LinkerFlavor, Lld, get_linker},
    spec::{Os, Target, TargetEnv},
};
use tempfile::NamedTempFile;

use args::*;

mod args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.nested {
        CaracSubcommand::Build(build) => {
            let BuildCommand {
                input_file,
                output_file,
                emit,
                code_model,
                optimize_level,
                reloc_mode,
                release,
                crate_name,
                target,
            } = build;

            let target = if let Some(target) = target {
                if let Some(target) = Target::by_name(&target) {
                    target
                } else {
                    bail!("Invalid target")
                }
            } else {
                Target::default()
            };

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

            let file_table = FileTable::new();
            let file_id = file_table.register_file(input_file.clone())?;

            let ast = ParseContext::new(&file_table).parse(&CaraParser, file_id)?;
            let ast = simplify(crate_name.clone(), ast);

            let query_ctx = QueryContext::builder()
                .crate_name(crate_name)
                .ast(Arc::new(ast))
                .target(target)
                .build();

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
                let flavor = match target.os {
                    Os::Windows => match target.env {
                        TargetEnv::Msvc => LinkerFlavor::Msvc(Lld::No),
                        _ => LinkerFlavor::Gnu(Cc::Yes, Lld::No),
                    },
                    Os::None => LinkerFlavor::Gnu(Cc::No, Lld::Yes),
                    Os::MacOs => LinkerFlavor::Darwin(Cc::Yes, Lld::No),
                    _ => LinkerFlavor::Gnu(Cc::Yes, Lld::No),
                };

                let mut linker = get_linker(
                    Path::new(if flavor.uses_cc() {
                        "cc"
                    } else if flavor.uses_lld() {
                        "lld"
                    } else {
                        "ld"
                    }),
                    flavor,
                    target,
                );
                linker.add_object(temp_file.path());
                linker.output_filename(Path::new(&output_file));
                if matches!(target.os, Os::None) {
                    linker.set_no_stdlib();
                }

                let child = linker.cmd();

                if !child.command().spawn()?.wait()?.success() {
                    anyhow::bail!("Failed to link executable");
                }
            }
        }
    }

    Ok(())
}
