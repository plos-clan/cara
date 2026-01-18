use std::str::FromStr;

mod targets;

#[derive(Debug, Clone, Copy)]
pub struct Target {
    pub arch: Arch,
    pub os: Os,
    pub env: TargetEnv,
}

impl Target {
    pub fn pointer_width(&self) -> u32 {
        match self.arch {
            Arch::X86_64 => 64,
            Arch::Aarch64 => 64,
        }
    }
}

super::target_spec_enum! {
    pub enum Arch {
        X86_64 = "x86_64",
        Aarch64 = "aarch64",
    }

    parse_error_type = "arch";
}

super::target_spec_enum! {
    pub enum Os {
        Linux = "linux",
        Windows = "windows",
        MacOs = "macos",
        None = "none",
    }

    parse_error_type = "os";
}

super::target_spec_enum! {
    pub enum TargetEnv {
        Msvc = "msvc",
        Gnu = "gnu",
        Musl = "musl",
        Unspecified = "",
        MacAbi = "macabi",
    }

    parse_error_type = "env";
}
