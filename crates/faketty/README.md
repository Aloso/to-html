# faketty

Rust library to run a command in bash, pretending to be a tty. This means that the command will assume that terminal colors and other terminal features are available.

Note that some programs might still behave differently than they would in a real terminal. For example, on my system, `ls` always displays colors in the terminal, but requires `--color=auto` when executed in faketty.

## Example

```rust
let output = faketty::bash_command("ls --color=auto").output().unwrap();
assert!(output.status.success());

let _stdout: String = String::from_utf8(output.stdout.to_vec())
    .expect("Invalid UTF-8");
```
