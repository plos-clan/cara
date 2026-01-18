use std::str::FromStr;

/// Linker is called through a C/C++ compiler.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Cc {
    Yes,
    No,
}

/// Linker is LLD.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Lld {
    Yes,
    No,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LinkerFlavor {
    Gnu(Cc, Lld),
    Darwin(Cc, Lld),
    Unix(Cc),
    Msvc(Lld),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LinkerFlavorCli {
    // Modern (unstable) flavors, with direct counterparts in `LinkerFlavor`.
    Gnu(Cc, Lld),
    Darwin(Cc, Lld),
    Unix(Cc),
    // Note: `Msvc(Lld::No)` is also a stable value.
    Msvc(Lld),

    // Legacy stable values
    Gcc,
    Ld,
    Lld(LldFlavor),
}

impl LinkerFlavorCli {
    /// Returns whether this `-C linker-flavor` option is one of the unstable values.
    pub fn is_unstable(&self) -> bool {
        match self {
            LinkerFlavorCli::Gnu(..)
            | LinkerFlavorCli::Darwin(..)
            | LinkerFlavorCli::Unix(..)
            | LinkerFlavorCli::Msvc(Lld::Yes) => true,
            LinkerFlavorCli::Gcc
            | LinkerFlavorCli::Ld
            | LinkerFlavorCli::Lld(..)
            | LinkerFlavorCli::Msvc(Lld::No) => false,
        }
    }
}

crate::target_spec_enum! {
    pub enum LldFlavor {
        Ld64 = "darwin",
        Ld = "gnu",
        Link = "link",
    }

    parse_error_type = "LLD flavor";
}

impl LinkerFlavor {
    /// At this point the target's reference linker flavor doesn't yet exist and we need to infer
    /// it. The inference always succeeds and gives some result, and we don't report any flavor
    /// incompatibility errors for json target specs. The CLI flavor is used as the main source
    /// of truth, other flags are used in case of ambiguities.
    fn from_cli_json(cli: LinkerFlavorCli, lld_flavor: LldFlavor, is_gnu: bool) -> LinkerFlavor {
        match cli {
            LinkerFlavorCli::Gnu(cc, lld) => LinkerFlavor::Gnu(cc, lld),
            LinkerFlavorCli::Darwin(cc, lld) => LinkerFlavor::Darwin(cc, lld),
            LinkerFlavorCli::Msvc(lld) => LinkerFlavor::Msvc(lld),
            LinkerFlavorCli::Unix(cc) => LinkerFlavor::Unix(cc),

            // Below: legacy stable values
            LinkerFlavorCli::Gcc => match lld_flavor {
                LldFlavor::Ld if is_gnu => LinkerFlavor::Gnu(Cc::Yes, Lld::No),
                LldFlavor::Ld64 => LinkerFlavor::Darwin(Cc::Yes, Lld::No),
                LldFlavor::Ld | LldFlavor::Link => LinkerFlavor::Unix(Cc::Yes),
            },
            LinkerFlavorCli::Ld => match lld_flavor {
                LldFlavor::Ld if is_gnu => LinkerFlavor::Gnu(Cc::No, Lld::No),
                LldFlavor::Ld64 => LinkerFlavor::Darwin(Cc::No, Lld::No),
                LldFlavor::Ld | LldFlavor::Link => LinkerFlavor::Unix(Cc::No),
            },
            LinkerFlavorCli::Lld(LldFlavor::Ld) => LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            LinkerFlavorCli::Lld(LldFlavor::Ld64) => LinkerFlavor::Darwin(Cc::No, Lld::Yes),
            LinkerFlavorCli::Lld(LldFlavor::Link) => LinkerFlavor::Msvc(Lld::Yes),
        }
    }

    /// Returns the corresponding backwards-compatible CLI flavor.
    fn to_cli(self) -> LinkerFlavorCli {
        match self {
            LinkerFlavor::Gnu(Cc::Yes, _)
            | LinkerFlavor::Darwin(Cc::Yes, _)
            | LinkerFlavor::Unix(Cc::Yes) => LinkerFlavorCli::Gcc,
            LinkerFlavor::Gnu(_, Lld::Yes) => LinkerFlavorCli::Lld(LldFlavor::Ld),
            LinkerFlavor::Darwin(_, Lld::Yes) => LinkerFlavorCli::Lld(LldFlavor::Ld64),
            LinkerFlavor::Gnu(..) | LinkerFlavor::Darwin(..) | LinkerFlavor::Unix(..) => {
                LinkerFlavorCli::Ld
            }
            LinkerFlavor::Msvc(Lld::Yes) => LinkerFlavorCli::Lld(LldFlavor::Link),
            LinkerFlavor::Msvc(..) => LinkerFlavorCli::Msvc(Lld::No),
        }
    }

    /// Returns the modern CLI flavor that is the counterpart of this flavor.
    fn to_cli_counterpart(self) -> LinkerFlavorCli {
        match self {
            LinkerFlavor::Gnu(cc, lld) => LinkerFlavorCli::Gnu(cc, lld),
            LinkerFlavor::Darwin(cc, lld) => LinkerFlavorCli::Darwin(cc, lld),
            LinkerFlavor::Unix(cc) => LinkerFlavorCli::Unix(cc),
            LinkerFlavor::Msvc(lld) => LinkerFlavorCli::Msvc(lld),
        }
    }

    fn infer_cli_hints(cli: LinkerFlavorCli) -> (Option<Cc>, Option<Lld>) {
        match cli {
            LinkerFlavorCli::Gnu(cc, lld) | LinkerFlavorCli::Darwin(cc, lld) => {
                (Some(cc), Some(lld))
            }
            LinkerFlavorCli::Unix(cc) => (Some(cc), None),
            LinkerFlavorCli::Msvc(lld) => (Some(Cc::No), Some(lld)),

            // Below: legacy stable values
            LinkerFlavorCli::Gcc => (Some(Cc::Yes), None),
            LinkerFlavorCli::Ld => (Some(Cc::No), Some(Lld::No)),
            LinkerFlavorCli::Lld(_) => (Some(Cc::No), Some(Lld::Yes)),
        }
    }

    fn infer_linker_hints(linker_stem: &str) -> Result<Self, (Option<Cc>, Option<Lld>)> {
        // Remove any version postfix.
        let stem = linker_stem
            .rsplit_once('-')
            .and_then(|(lhs, rhs)| rhs.chars().all(char::is_numeric).then_some(lhs))
            .unwrap_or(linker_stem);

        if stem == "emcc" // GCC/Clang can have an optional target prefix.
            || stem == "gcc"
            || stem.ends_with("-gcc")
            || stem == "g++"
            || stem.ends_with("-g++")
            || stem == "clang"
            || stem.ends_with("-clang")
            || stem == "clang++"
            || stem.ends_with("-clang++")
        {
            Err((Some(Cc::Yes), Some(Lld::No)))
        } else if stem == "wasm-ld"
            || stem.ends_with("-wasm-ld")
            || stem == "ld.lld"
            || stem == "lld"
            || stem == "rust-lld"
            || stem == "lld-link"
        {
            Err((Some(Cc::No), Some(Lld::Yes)))
        } else if stem == "ld" || stem.ends_with("-ld") || stem == "link" {
            Err((Some(Cc::No), Some(Lld::No)))
        } else {
            Err((None, None))
        }
    }

    fn with_hints(self, (cc_hint, lld_hint): (Option<Cc>, Option<Lld>)) -> LinkerFlavor {
        match self {
            LinkerFlavor::Gnu(cc, lld) => {
                LinkerFlavor::Gnu(cc_hint.unwrap_or(cc), lld_hint.unwrap_or(lld))
            }
            LinkerFlavor::Darwin(cc, lld) => {
                LinkerFlavor::Darwin(cc_hint.unwrap_or(cc), lld_hint.unwrap_or(lld))
            }
            LinkerFlavor::Unix(cc) => LinkerFlavor::Unix(cc_hint.unwrap_or(cc)),
            LinkerFlavor::Msvc(lld) => LinkerFlavor::Msvc(lld_hint.unwrap_or(lld)),
        }
    }

    pub fn with_cli_hints(self, cli: LinkerFlavorCli) -> LinkerFlavor {
        self.with_hints(LinkerFlavor::infer_cli_hints(cli))
    }

    pub fn with_linker_hints(self, linker_stem: &str) -> LinkerFlavor {
        match LinkerFlavor::infer_linker_hints(linker_stem) {
            Ok(linker_flavor) => linker_flavor,
            Err(hints) => self.with_hints(hints),
        }
    }

    pub fn check_compatibility(self, cli: LinkerFlavorCli) -> Option<String> {
        let compatible = |cli| {
            // The CLI flavor should be compatible with the target if:
            match (self, cli) {
                // 1. they are counterparts: they have the same principal flavor.
                (LinkerFlavor::Gnu(..), LinkerFlavorCli::Gnu(..))
                | (LinkerFlavor::Darwin(..), LinkerFlavorCli::Darwin(..))
                | (LinkerFlavor::Unix(..), LinkerFlavorCli::Unix(..))
                | (LinkerFlavor::Msvc(..), LinkerFlavorCli::Msvc(..)) => return true,
                // 2. The linker flavor is independent of target and compatible
                _ => {}
            }

            // 3. or, the flavor is legacy and survives this roundtrip.
            cli == self.with_cli_hints(cli).to_cli()
        };
        (!compatible(cli)).then(|| {
            LinkerFlavorCli::all()
                .iter()
                .filter(|cli| compatible(**cli))
                .map(|cli| cli.desc())
                .intersperse(", ")
                .collect()
        })
    }

    pub fn lld_flavor(self) -> LldFlavor {
        match self {
            LinkerFlavor::Gnu(..) | LinkerFlavor::Unix(..) => LldFlavor::Ld,
            LinkerFlavor::Darwin(..) => LldFlavor::Ld64,
            LinkerFlavor::Msvc(..) => LldFlavor::Link,
        }
    }

    pub fn is_gnu(self) -> bool {
        matches!(self, LinkerFlavor::Gnu(..))
    }

    /// Returns whether the flavor uses the `lld` linker.
    pub fn uses_lld(self) -> bool {
        // Exhaustive match in case new flavors are added in the future.
        match self {
            LinkerFlavor::Gnu(_, Lld::Yes)
            | LinkerFlavor::Darwin(_, Lld::Yes)
            | LinkerFlavor::Msvc(Lld::Yes) => true,
            LinkerFlavor::Gnu(..)
            | LinkerFlavor::Darwin(..)
            | LinkerFlavor::Msvc(_)
            | LinkerFlavor::Unix(_) => false,
        }
    }

    /// Returns whether the flavor calls the linker via a C/C++ compiler.
    pub fn uses_cc(self) -> bool {
        // Exhaustive match in case new flavors are added in the future.
        match self {
            LinkerFlavor::Gnu(Cc::Yes, _)
            | LinkerFlavor::Darwin(Cc::Yes, _)
            | LinkerFlavor::Unix(Cc::Yes) => true,
            LinkerFlavor::Gnu(..)
            | LinkerFlavor::Darwin(..)
            | LinkerFlavor::Msvc(_)
            | LinkerFlavor::Unix(_) => false,
        }
    }

    /// For flavors with an `Lld` component, ensure it's enabled. Otherwise, returns the given
    /// flavor unmodified.
    pub fn with_lld_enabled(self) -> LinkerFlavor {
        match self {
            LinkerFlavor::Gnu(cc, Lld::No) => LinkerFlavor::Gnu(cc, Lld::Yes),
            LinkerFlavor::Darwin(cc, Lld::No) => LinkerFlavor::Darwin(cc, Lld::Yes),
            LinkerFlavor::Msvc(Lld::No) => LinkerFlavor::Msvc(Lld::Yes),
            _ => self,
        }
    }

    /// For flavors with an `Lld` component, ensure it's disabled. Otherwise, returns the given
    /// flavor unmodified.
    pub fn with_lld_disabled(self) -> LinkerFlavor {
        match self {
            LinkerFlavor::Gnu(cc, Lld::Yes) => LinkerFlavor::Gnu(cc, Lld::No),
            LinkerFlavor::Darwin(cc, Lld::Yes) => LinkerFlavor::Darwin(cc, Lld::No),
            LinkerFlavor::Msvc(Lld::Yes) => LinkerFlavor::Msvc(Lld::No),
            _ => self,
        }
    }
}

macro_rules! linker_flavor_cli_impls {
    ($(($($flavor:tt)*) $string:literal)*) => (
        impl LinkerFlavorCli {
            const fn all() -> &'static [LinkerFlavorCli] {
                &[$($($flavor)*,)*]
            }

            pub const fn one_of() -> &'static str {
                concat!("one of: ", $($string, " ",)*)
            }

            pub fn desc(self) -> &'static str {
                match self {
                    $($($flavor)* => $string,)*
                }
            }
        }

        impl FromStr for LinkerFlavorCli {
            type Err = String;

            fn from_str(s: &str) -> Result<LinkerFlavorCli, Self::Err> {
                Ok(match s {
                    $($string => $($flavor)*,)*
                    _ => return Err(format!("invalid linker flavor, allowed values: {}", Self::one_of())),
                })
            }
        }
    )
}

linker_flavor_cli_impls! {
    (LinkerFlavorCli::Gnu(Cc::No, Lld::No)) "gnu"
    (LinkerFlavorCli::Gnu(Cc::No, Lld::Yes)) "gnu-lld"
    (LinkerFlavorCli::Gnu(Cc::Yes, Lld::No)) "gnu-cc"
    (LinkerFlavorCli::Gnu(Cc::Yes, Lld::Yes)) "gnu-lld-cc"
    (LinkerFlavorCli::Darwin(Cc::No, Lld::No)) "darwin"
    (LinkerFlavorCli::Darwin(Cc::No, Lld::Yes)) "darwin-lld"
    (LinkerFlavorCli::Darwin(Cc::Yes, Lld::No)) "darwin-cc"
    (LinkerFlavorCli::Darwin(Cc::Yes, Lld::Yes)) "darwin-lld-cc"
    (LinkerFlavorCli::Unix(Cc::No)) "unix"
    (LinkerFlavorCli::Unix(Cc::Yes)) "unix-cc"
    (LinkerFlavorCli::Msvc(Lld::Yes)) "msvc-lld"
    (LinkerFlavorCli::Msvc(Lld::No)) "msvc"

    // Legacy stable flavors
    (LinkerFlavorCli::Gcc) "gcc"
    (LinkerFlavorCli::Ld) "ld"
    (LinkerFlavorCli::Lld(LldFlavor::Ld)) "ld.lld"
    (LinkerFlavorCli::Lld(LldFlavor::Ld64)) "ld64.lld"
    (LinkerFlavorCli::Lld(LldFlavor::Link)) "lld-link"
}
