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

/// Builds the cara file.
#[derive(FromArgs)]
#[argh(subcommand, name = "build")]
pub struct BuildCommand {
    #[argh(positional)]
    pub input_file: String,
    /// the output path.
    #[argh(option, default = "String::from(\"a.out\")", short = 'o')]
    pub output_file: String,
    /// the file type emitted.
    #[argh(option, default = "BuildResult::Executable")]
    pub emit: BuildResult,
}
