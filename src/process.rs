/// Get the command of an ancestor process. `level` specifies how far up the process tree we should go:
/// - 0 means the current process
/// - 1 means the parent process
/// - 2 means the grand-parent process
/// - etc.
///
/// The resulting string is either a program name, such as `bash`, or a path to an executable, e.g.
/// `/usr/lib/firefox/firefox`.
///
/// Any error results in `None` being returned. For example, this happens if you go beyond the root process
/// (pid=1) in the hierarchy.
pub(super) fn get_ancestor_process_cmd(level: u32) -> Option<String> {
    let mut pid = unsafe { libc::getpid() };
    for _ in 0..level {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        pid = stat.split(' ').nth(3)?.parse().ok()?;
    }

    let mut name = std::fs::read_to_string(format!("/proc/{pid}/cmdline")).ok()?;
    if name.ends_with('\0') {
        let _ = name.pop();
    }
    Some(name)
}
