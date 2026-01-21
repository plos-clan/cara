# The Cara Programming Language

Cara is a kind of simple, statically typed programming language, inspired by Rust and Zig.
Currently in development.

## Example
[main.cara](main.cara)
You can build this with the following command:
``` sh
carac build main.cara -o main.bin
```
Then you can execute it:
``` sh
./main.bin
```
> [!NOTE]
> main.cara depends on [module.cara](module.cara).

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
