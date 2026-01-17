use std::str::FromStr;

mod targets;

#[derive(Debug, Clone, Copy)]
pub struct Target {
    pub arch: Arch,
    pub os: Os,
    pub env: TargetEnv,
    pub pointer_width: u16,
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
