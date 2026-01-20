use clap::{Parser, Subcommand, ValueEnum, crate_version};

/// Cara compiler
#[derive(Debug, Parser)]
#[clap(version = crate_version!())]
#[clap(about)]
pub struct Args {
    #[clap(subcommand)]
    pub nested: CaracSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum CaracSubcommand {
    Build(BuildCommand),
}

/// Builds the cara file.
#[derive(Debug, Parser)]
pub struct BuildCommand {
    /// the input file.
    pub input_file: String,
    /// set if build in release mode.
    #[arg(short, long)]
    pub release: bool,
    /// the output path.
    #[arg(short, long, default_value = "a.out")]
    pub output_file: String,
    /// the file type emitted.
    #[arg(long, value_enum, default_value = "exe")]
    pub emit: BuildResult,
    /// code model.
    #[arg(long, value_enum, default_value = "default")]
    pub code_model: CodeModel,
    /// optimization level.
    #[arg(long, value_enum, default_value = "O2")]
    pub optimize_level: OptimizeLevel,
    /// relocation mode.
    #[arg(long, value_enum, default_value = "default")]
    pub reloc_mode: RelocMode,
    /// crate name.
    #[arg(long, default_value = r#"main"#)]
    pub crate_name: String,
    /// target triple.
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum BuildResult {
    /// Emit LLVM IR.
    Ir,
    /// Emit assembly.
    Asm,
    /// Emit object file.
    #[value(name = "obj")]
    Object,
    /// Emit executable.
    #[value(name = "exe")]
    Executable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CodeModel {
    Large,
    Medium,
    Small,
    Kernel,
    Default,
}

impl From<CodeModel> for codegen::CodeModel {
    fn from(value: CodeModel) -> Self {
        match value {
            CodeModel::Default => Self::Default,
            CodeModel::Large => Self::Large,
            CodeModel::Medium => Self::Medium,
            CodeModel::Small => Self::Small,
            CodeModel::Kernel => Self::Kernel,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[value(rename_all = "UPPER")]
pub enum OptimizeLevel {
    O0,
    O1,
    O2,
    O3,
}

impl From<OptimizeLevel> for codegen::OptimizeLevel {
    fn from(value: OptimizeLevel) -> Self {
        match value {
            OptimizeLevel::O0 => Self::O0,
            OptimizeLevel::O1 => Self::O1,
            OptimizeLevel::O2 => Self::O2,
            OptimizeLevel::O3 => Self::O3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum RelocMode {
    Default,
    Static,
    Pic,
    DynamicNoPic,
}

impl From<RelocMode> for codegen::RelocMode {
    fn from(value: RelocMode) -> Self {
        match value {
            RelocMode::Default => Self::Pic,
            RelocMode::Static => Self::Static,
            RelocMode::Pic => Self::Pic,
            RelocMode::DynamicNoPic => Self::DynamicNoPic,
        }
    }
}
