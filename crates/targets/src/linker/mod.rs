use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::{env, iter, mem};

use find_msvc_tools;

use command::Command;
pub use flavor::*;

use crate::spec::{Target, TargetEnv};

mod command;
mod flavor;

/// Disables non-English messages from localized linkers.
/// Such messages may cause issues with text encoding on Windows (#35785)
/// and prevent inspection of linker output in case of errors, which we occasionally do.
/// This should be acceptable because other messages from rustc are in English anyway,
/// and may also be desirable to improve searchability of the linker diagnostics.
pub fn disable_localization(linker: &mut Command) {
    // No harm in setting both env vars simultaneously.
    // Unix-style linkers.
    linker.env("LC_ALL", "C");
    // MSVC's `link.exe`.
    linker.env("VSLANG", "1033");
}

/// The third parameter is for env vars, used on windows to set up the
/// path for MSVC to find its DLLs, and gcc to find its bundled
/// toolchain
pub fn get_linker<'a>(linker: &Path, flavor: LinkerFlavor, target: Target) -> Box<dyn Linker + 'a> {
    let msvc_tool = find_msvc_tools::find_tool(target.arch.desc(), "link.exe");

    let mut cmd = match linker.to_str() {
        Some(linker) if cfg!(windows) && linker.ends_with(".bat") => Command::bat_script(linker),
        _ => match flavor {
            LinkerFlavor::Gnu(Cc::No, Lld::Yes)
            | LinkerFlavor::Darwin(Cc::No, Lld::Yes)
            | LinkerFlavor::Msvc(Lld::Yes) => Command::lld(linker, flavor.lld_flavor()),
            LinkerFlavor::Msvc(Lld::No) => {
                Command::new(msvc_tool.as_ref().map_or(linker, |t| t.path()))
            }
            _ => Command::new(linker),
        },
    };

    if target.env == TargetEnv::Msvc
        && let Some(ref tool) = msvc_tool
    {
        for (k, v) in tool.env() {
            cmd.env(k, v);
        }
    }

    cmd.env("PATH", env::var("PATH").unwrap());

    // FIXME: Move `/LIBPATH` addition for uwp targets from the linker construction
    // to the linker args construction.
    assert!(cmd.get_args().is_empty());
    match flavor {
        LinkerFlavor::Gnu(cc, _) | LinkerFlavor::Darwin(cc, _) | LinkerFlavor::Unix(cc) => {
            Box::new(GccLinker {
                cmd,
                is_ld: cc == Cc::No,
            }) as Box<dyn Linker>
        }
        LinkerFlavor::Msvc(..) => Box::new(MsvcLinker { cmd }) as Box<dyn Linker>,
    }
}

// Note: Ideally neither these helper function, nor the macro-generated inherent methods below
// would exist, and these functions would live in `trait Linker`.
// Unfortunately, adding these functions to `trait Linker` make it `dyn`-incompatible.
// If the methods are added to the trait with `where Self: Sized` bounds, then even a separate
// implementation of them for `dyn Linker {}` wouldn't work due to a conflict with those
// uncallable methods in the trait.

/// Just pass the arguments to the linker as is.
/// It is assumed that they are correctly prepared in advance.
fn verbatim_args<L: Linker + ?Sized>(
    l: &mut L,
    args: impl IntoIterator<Item: AsRef<OsStr>>,
) -> &mut L {
    for arg in args {
        l.cmd().arg(arg);
    }
    l
}
/// Add underlying linker arguments to C compiler command, by wrapping them in
/// `-Wl` or `-Xlinker`.
fn convert_link_args_to_cc_args(cmd: &mut Command, args: impl IntoIterator<Item: AsRef<OsStr>>) {
    let mut combined_arg = OsString::from("-Wl");
    for arg in args {
        // If the argument itself contains a comma, we need to emit it
        // as `-Xlinker`, otherwise we can use `-Wl`.
        if arg.as_ref().as_encoded_bytes().contains(&b',') {
            // Emit current `-Wl` argument, if any has been built.
            if combined_arg != OsStr::new("-Wl") {
                cmd.arg(combined_arg);
                // Begin next `-Wl` argument.
                combined_arg = OsString::from("-Wl");
            }

            // Emit `-Xlinker` argument.
            cmd.arg("-Xlinker");
            cmd.arg(arg);
        } else {
            // Append to `-Wl` argument.
            combined_arg.push(",");
            combined_arg.push(arg);
        }
    }
    // Emit final `-Wl` argument.
    if combined_arg != OsStr::new("-Wl") {
        cmd.arg(combined_arg);
    }
}
/// Arguments for the underlying linker.
/// Add options to pass them through cc wrapper if `Linker` is a cc wrapper.
fn link_args<L: Linker + ?Sized>(l: &mut L, args: impl IntoIterator<Item: AsRef<OsStr>>) -> &mut L {
    if !l.is_cc() {
        verbatim_args(l, args);
    } else {
        convert_link_args_to_cc_args(l.cmd(), args);
    }
    l
}
/// Arguments for the cc wrapper specifically.
/// Check that it's indeed a cc wrapper and pass verbatim.
fn cc_args<L: Linker + ?Sized>(l: &mut L, args: impl IntoIterator<Item: AsRef<OsStr>>) -> &mut L {
    assert!(l.is_cc());
    verbatim_args(l, args)
}
/// Arguments supported by both underlying linker and cc wrapper, pass verbatim.
fn link_or_cc_args<L: Linker + ?Sized>(
    l: &mut L,
    args: impl IntoIterator<Item: AsRef<OsStr>>,
) -> &mut L {
    verbatim_args(l, args)
}

macro_rules! generate_arg_methods {
    ($($ty:ty)*) => { $(
        impl $ty {
            #[allow(unused)]
            pub(crate) fn verbatim_args(&mut self, args: impl IntoIterator<Item: AsRef<OsStr>>) -> &mut Self {
                verbatim_args(self, args)
            }
            #[allow(unused)]
            pub(crate) fn verbatim_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
                verbatim_args(self, iter::once(arg))
            }
            #[allow(unused)]
            pub(crate) fn link_args(&mut self, args: impl IntoIterator<Item: AsRef<OsStr>>) -> &mut Self {
                link_args(self, args)
            }
            #[allow(unused)]
            pub(crate) fn link_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
                link_args(self, iter::once(arg))
            }
            #[allow(unused)]
            pub(crate) fn cc_args(&mut self, args: impl IntoIterator<Item: AsRef<OsStr>>) -> &mut Self {
                cc_args(self, args)
            }
            #[allow(unused)]
            pub(crate) fn cc_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
                cc_args(self, iter::once(arg))
            }
            #[allow(unused)]
            pub(crate) fn link_or_cc_args(&mut self, args: impl IntoIterator<Item: AsRef<OsStr>>) -> &mut Self {
                link_or_cc_args(self, args)
            }
            #[allow(unused)]
            pub(crate) fn link_or_cc_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
                link_or_cc_args(self, iter::once(arg))
            }
        }
    )* }
}

generate_arg_methods! {
    GccLinker
    MsvcLinker
    dyn Linker + '_
}

/// Linker abstraction used by `back::link` to build up the command to invoke a
/// linker.
///
/// This trait is the total list of requirements needed by `back::link` and
/// represents the meaning of each option being passed down. This trait is then
/// used to dispatch on whether a GNU-like linker (generally `ld.exe`) or an
/// MSVC linker (e.g., `link.exe`) is being used.
pub trait Linker {
    fn cmd(&mut self) -> &mut Command;
    fn is_cc(&self) -> bool {
        false
    }
    fn output_filename(&mut self, path: &Path) {
        link_or_cc_args(link_or_cc_args(self, &["-o"]), &[path]);
    }
    fn add_object(&mut self, path: &Path) {
        link_or_cc_args(self, &[path]);
    }
    fn set_no_stdlib(&mut self) {
        link_or_cc_args(self, &["-nostdlib"]);
    }
}

impl dyn Linker + '_ {
    pub fn take_cmd(&mut self) -> Command {
        mem::replace(self.cmd(), Command::new(""))
    }
}

struct GccLinker {
    cmd: Command,
    // Link as ld
    is_ld: bool,
}

impl Linker for GccLinker {
    fn cmd(&mut self) -> &mut Command {
        &mut self.cmd
    }

    fn is_cc(&self) -> bool {
        !self.is_ld
    }
}

struct MsvcLinker {
    cmd: Command,
}

impl Linker for MsvcLinker {
    fn cmd(&mut self) -> &mut Command {
        &mut self.cmd
    }
}
