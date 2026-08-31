fn run(s: &str) -> String {
    let output = crate::bash_command(s).unwrap().output().unwrap();
    let s1 = crate::get_stdout(output.stdout).unwrap();

    if crate::which_shell("zsh").is_ok() {
        let output = crate::command(s, Some("zsh")).unwrap().output().unwrap();
        let s2 = crate::get_stdout(output.stdout).unwrap();

        assert_eq!(s1, s2);
    }

    s1
}

#[test]
fn echo() {
    assert_eq!(run("echo hello world"), "hello world\n");
}

#[test]
fn seq() {
    assert_eq!(run("seq 3"), "1\n2\n3\n");
}

#[test]
fn echo_quotes() {
    assert_eq!(run(r#"echo "Hello \$\`' world!""#), "Hello $`' world!\n");
}

#[test]
fn echo_and_cat() {
    assert_eq!(
        run("echo 'look, bash support!' | cat"),
        "look, bash support!\n"
    );
}
