/*
* @file    :   main.rs
* @time    :   2024/03/30 10:51:44
* @author  :   zzjcarrot
*/

use std::{cell::RefCell, io::Read, path::Path, rc::Rc, sync::{Arc, RwLock}};

use argh::FromArgs;
use cara::backend::GenerateProgramTwice;
use inkwell::{
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};

#[derive(FromArgs)]
#[argh(description = "Cara compiler")]
struct CCPMArgs {
    #[argh(option, short = 'i')]
    #[argh(description = "the input source code")]
    input_source: String,

    #[argh(option, short = 'o')]
    #[argh(description = "the output file")]
    output_source: String,
}

fn main() {
    Target::initialize_all(&InitializationConfig::default());
    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::Aggressive,
            RelocMode::PIC,
            CodeModel::Small,
        )
        .unwrap();

    let mut code = String::new();

    let args: CCPMArgs = argh::from_env();

    let input_source = Path::new(args.input_source.as_str());

    if !args.input_source.ends_with(".cara") {
        panic!("Input file is not a cara source file!");
    }

    let mut input = std::fs::File::open(input_source).unwrap();

    input.read_to_string(&mut code).unwrap();

    let name = args.input_source.split('/').nth(0).unwrap().to_string();
    let name = name.split('.').nth(0).unwrap().to_string();

    let parser = cara::front::CParser::new(code,args.input_source.clone());

    let unit = parser.parse();
    //println!("{:#?}", unit);

    let context = Context::create();
    let module = context.create_module(name.clone().as_str());
    let builder = context.create_builder();

    let gen = cara::backend::Generator::new(name, &context, module, builder);

    let gen = Arc::new(RwLock::new(gen));
    gen.read().unwrap().prepare(&target_machine);

    unit.decl(gen.clone()).unwrap();
    unit.implement(gen.clone()).unwrap();

    for error in gen.read().unwrap().errors.iter() {
        println!("{:?}", error);
    }
    if gen.read().unwrap().errors.len() > 0 {
        return;
    }

    let gen = gen.read().unwrap();

    gen.module.print_to_stderr();

    gen.run_passes(&target_machine);

    gen.module.print_to_stderr();

    let path = Path::new(args.output_source.as_str());

    target_machine
        .write_to_file(&gen.module, FileType::Object, path)
        .unwrap();

    target_machine
        .write_to_file(
            &gen.module,
            FileType::Assembly,
            Path::new("test.asm"),
        )
        .unwrap();

}
