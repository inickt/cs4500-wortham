# xyes

`./xyes [-limit] [arguments...]`

The xyes program is used to output the given series of arguments,
or "hello world" if there are no arguments, forever.

The `-limit` option restricts the amount of lines displayed to 20.

### Editing, Building, and Running

The project is written in Rust and uses the `cargo` utility to
build and run the program. The source code is in `src/main.rs`.

If you've [installed Rust and Cargo](https://www.rust-lang.org/tools/install)
you can use `cargo build` and `cargo run -- [arguments...]` to build
and run the code in `main.rs` respectively. Run commands from the `Other` directory.

### Testing

The project is written in Rust and uses the `cargo` utility to
run tests. If you've [installed Rust and Cargo](https://www.rust-lang.org/tools/install), use `cargo test`
to run the test harness. Run commands from the `Other` directory. The test code itself is located 
in `main.rs` below the source code.