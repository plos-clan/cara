use argh::{FromArgValue, FromArgs};

/// Cara compiler
#[derive(FromArgs)]
pub struct Args {
    #[argh(subcommand)]
    pub nested: Subcommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Subcommand {
    Build(BuildCommand),
}

/// Builds the cara file.
#[derive(FromArgs)]
#[argh(subcommand, name = "build")]
pub struct BuildCommand {
    #[argh(positional)]
    pub input_file: String,
    /// set if build in release mode.
    #[argh(switch)]
    pub release: bool,
    /// the output path.
    #[argh(option, default = "String::from(\"a.out\")", short = 'o')]
    pub output_file: String,
    /// the file type emitted.
    #[argh(option, default = "BuildResult::Executable")]
    pub emit: BuildResult,
    /// code model.
    #[argh(option, default = "CodeModel::Default")]
    pub code_model: CodeModel,
    /// optimization level.
    #[argh(option, default = "OptimizeLevel::O2", short = 'O')]
    pub optimize_level: OptimizeLevel,
    /// relocation mode.
    #[argh(option, default = "RelocMode::Default")]
    pub reloc_mode: RelocMode,
    /// crate name.
    #[argh(option, default = r#""main".into()"#)]
    pub crate_name: String,
    /// target triple.
    #[argh(option)]
    pub target: Option<String>,
}

pub enum BuildResult {
    Ir,
    Asm,
    Object,
    Executable,
}

impl FromArgValue for BuildResult {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        Ok(match value {
            "ir" => Self::Ir,
            "asm" => Self::Asm,
            "obj" => Self::Object,
            "exe" => Self::Executable,
            _ => return Err(format!("Invalid build result type: {}", value)),
        })
    }
}

pub enum CodeModel {
    Large,
    Medium,
    Small,
    Kernel,
    Default,
}

impl FromArgValue for CodeModel {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        Ok(match value {
            "large" => Self::Large,
            "small" => Self::Small,
            "kernel" => Self::Kernel,
            "default" => Self::Default,
            _ => return Err(format!("Invalid code model: {}", value)),
        })
    }
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

pub enum OptimizeLevel {
    O0,
    O1,
    O2,
    O3,
}

impl FromArgValue for OptimizeLevel {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        Ok(match value {
            "0" => Self::O0,
            "1" => Self::O1,
            "2" => Self::O2,
            "3" => Self::O3,
            _ => return Err(format!("Invalid optimize level: {}", value)),
        })
    }
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

#[derive(Debug, Clone, Copy)]
pub enum RelocMode {
    Default,
    Static,
    Pic,
    DynamicNoPic,
}

impl FromArgValue for RelocMode {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        Ok(match value {
            "default" => Self::Default,
            "static" => Self::Static,
            "pic" => Self::Pic,
            "dynamic-nopic" => Self::DynamicNoPic,
            _ => return Err(format!("Invalid reloc mode: {}", value)),
        })
    }
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
