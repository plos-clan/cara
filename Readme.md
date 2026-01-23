# The Cara Programming Language

Cara is a kind of simple, statically typed programming language, inspired by Rust and Zig.
Currently in development.

## Install

Download the latest release from [GitHub](https://github.com/plos-clan/cara/releases). \
For Windows(x86_64 only), download xxx-x86_64-pc-windows-msvc.zip. \
For Mac(Aarch64 only), download xxx-aarch64-apple-darwin.tar.gz. \
For Linux(x86_64 only), download xxx-x86_64-unknown-linux-gnu.tar.gz.

## Example
[main.cara](main.cara) \
You can build this with the following command:
``` sh
carac build main.cara -o main.bin
```
Then you can execute it:
``` sh
./main.bin
```
> [!NOTE]
> main.cara depends on [module.cara](module.cara), so if you want to compile it, you need to download module.cara and put it in the same directory as main.cara.

## Roadmap
- [x] Basic syntax and ast.
- [x] Basic const evaluation.
- [x] Query system driven compilation.
- [x] LLVM backend.
- [x] Type Casting.
- [x] Powerful analyzer.
- [ ] Structs and tuples.
- [x] Modules.
- [ ] OOP.
- [ ] Enums and matches.
- [ ] Generics.
- [ ] Light weight procedural macros.
- [ ] LSP.
- [ ] Incremental compilation
