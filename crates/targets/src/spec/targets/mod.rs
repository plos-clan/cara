use super::{Arch, Os, Target, TargetEnv};

macro_rules! targets {
    ($(
        $name: literal = ($arch: ident, $os: ident, $abi: ident)
    ),* $(,)?) => {
        static TARGETS: &[(&'static str, Target)] = &[
            $(
                ($name, Target {
                    arch: Arch::$arch,
                    os: Os::$os,
                    env: TargetEnv::$abi,
                })
            ),*
        ];
    };
}

targets! {
    "x86_64-linux-gnu" = (X86_64, Linux, Gnu),
    "x86_64-linux-musl" = (X86_64, Linux, Musl),

    "x86_64-windows-gnu" = (X86_64, Windows, Gnu),
    "x86_64-windows-msvc" = (X86_64, Windows, Msvc),

    "aarch64-macos-darwin" = (Aarch64, MacOs, Unspecified),
    "x86_64-apple-darwin" = (X86_64, MacOs, Unspecified),

    "x86_64-unknown-none" = (X86_64, None, Unspecified),
    "aarch64-unknown-none" = (Aarch64, None, Unspecified),
}

impl Target {
    pub fn by_name(name: &str) -> Option<Target> {
        TARGETS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, t)| t.clone())
    }
}

impl Default for Target {
    fn default() -> Self {
        let arch = if cfg!(target_arch = "x86_64") {
            Arch::X86_64
        } else if cfg!(target_arch = "aarch64") {
            Arch::Aarch64
        } else {
            panic!("Unsupported architecture.")
        };
        let env = if cfg!(target_env = "gnu") {
            TargetEnv::Gnu
        } else if cfg!(target_env = "musl") {
            TargetEnv::Musl
        } else if cfg!(target_env = "msvc") {
            TargetEnv::Msvc
        } else {
            TargetEnv::Unspecified
        };
        let os = if cfg!(target_os = "linux") {
            Os::Linux
        } else if cfg!(target_os = "windows") {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::MacOs
        } else {
            panic!("Unsupported operating system.")
        };
        Self { arch, os, env }
    }
}
