# fake-tty

[Documentation](https://docs.rs/crate/fake-tty)

Rust library to run a command in bash, pretending to be a tty. This means that the command will assume that terminal colors and other terminal features are available. This is done by executing the [script](https://man7.org/linux/man-pages/man1/script.1.html) command.

Note that some programs might still behave differently than they would in a real terminal. For example, on my system, `ls` always displays colors in the terminal, but requires `--color=auto` when executed in fake-tty.

## Example

```rust
let output = fake_tty::bash_command("ls --color=auto").output().unwrap();
assert!(output.status.success());

let _stdout: String = String::from_utf8(output.stdout).unwrap();
```

## Platform support

As of now, fake-tty supports Linux, macOS and FreeBSD.

Adding support other platforms should be easy, if they support bash and the `script` command. On Windows, it might be possible to use cmd or PowerShell instead; please send a pull request if you need Windows support.
