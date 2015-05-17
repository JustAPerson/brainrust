# Brainrust

Brainrust is a [Brainfuck] interpreter written in [Rust].

[Brainfuck]: https://en.wikipedia.org/wiki/Brainfuck
[Rust]: http://rust-lang.org/

# Building

Brainrust can be built using the [Rust nightly] binaries.

[Rust nightly]: http://www.rust-lang.org/install.html

```
$ git clone https://github.com/JustAPerson/brainrust.git
$ cd brainrust
$ cargo build
```

# Usage

Brainrust is pretty straightforward to use. If given a file name argument, then 
it will interpret that file's contents or else it will operate as a [REPL].

[REPL]: https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop

```
$ cat > hello_world.b
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>
---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
$ cargo run hello_world.b
Hello World!
```

## Caveats

The input operator `,` does not work correctly in the REPL mode because
Rust is currently unable to provide unbuffered access to stdin.

# License / Copyright

Brainrust is available under the terms of the MIT license.
See [LICENSE.md](./LICENSE.md) for details.
