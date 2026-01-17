use super::{Arch, Os, Target, TargetEnv};

macro_rules! targets {
    ($(
        $name: literal = ($arch: ident, $os: ident, $abi: ident, $pointer_width: literal)
    ),* $(,)?) => {
        static TARGETS: &[(&'static str, Target)] = &[
            $(
                ($name, Target {
                    arch: Arch::$arch,
                    os: Os::$os,
                    env: TargetEnv::$abi,
                    pointer_width: $pointer_width,
                })
            ),*
        ];
    };
}

targets! {
    "x86_64-linux-gnu" = (X86_64, Linux, Gnu, 64),
    "x86_64-linux-musl" = (X86_64, Linux, Musl, 64),
    "x86_64-windows-gnu" = (X86_64, Windows, Gnu, 64),
    "x86_64-windows-msvc" = (X86_64, Windows, Msvc, 64),
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
        let (pointer_width, arch) = {
            #[cfg(target_arch = "x86_64")]
            (64, Arch::X86_64)
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
        } else {
            panic!("Unsupported operating system.")
        };
        Self {
            arch,
            os,
            env,
            pointer_width,
        }
    }
}
