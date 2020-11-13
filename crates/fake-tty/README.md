# fake-tty

[Documentation](https://docs.rs/crate/fake-tty)

Rust library to run a command in bash, pretending to be a tty. This means that the command will assume that terminal colors and other terminal features are available.

Note that some programs might still behave differently than they would in a real terminal. For example, on my system, `ls` always displays colors in the terminal, but requires `--color=auto` when executed in fake-tty.

## Example

```rust
let output = fake_tty::bash_command("ls --color=auto").output().unwrap();
assert!(output.status.success());

let _stdout: String = String::from_utf8(output.stdout).unwrap();
```
