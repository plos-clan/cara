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

crate::target_spec_enum! {
    pub enum LldFlavor {
        Ld64 = "darwin",
        Ld = "gnu",
        Link = "link",
    }

    parse_error_type = "LLD flavor";
}

impl LinkerFlavor {
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
