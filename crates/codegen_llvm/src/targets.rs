use target_lexicon::{
    Aarch64Architecture, Architecture, BinaryFormat, Environment, OperatingSystem, Triple, Vendor,
};
use targets::spec::{Arch, Os, Target, TargetEnv};

pub fn llvm_target(value: Target) -> Triple {
    let arch = match value.arch {
        Arch::Aarch64 => Architecture::Aarch64(Aarch64Architecture::Aarch64),
        Arch::X86_64 => Architecture::X86_64,
    };
    let os = match value.os {
        Os::Linux => OperatingSystem::Linux,
        Os::MacOs => OperatingSystem::Darwin(None),
        Os::Windows => OperatingSystem::Windows,
        Os::None => OperatingSystem::None_,
    };
    let env = match value.env {
        TargetEnv::Gnu => Environment::Gnu,
        TargetEnv::Musl => Environment::Musl,
        TargetEnv::Unspecified => Environment::None,
        TargetEnv::MacAbi => Environment::Macabi,
        TargetEnv::Msvc => Environment::Msvc,
    };
    let binary_format = match value.os {
        Os::Linux => BinaryFormat::Elf,
        Os::MacOs => BinaryFormat::Macho,
        Os::Windows => BinaryFormat::Coff,
        Os::None => BinaryFormat::Elf,
    };
    Triple {
        architecture: arch,
        operating_system: os,
        environment: env,
        vendor: Vendor::Unknown,
        binary_format,
    }
}
