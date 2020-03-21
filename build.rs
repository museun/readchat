fn main() {
    println!(
        "cargo:rustc-env=GIT_REVISION={}",
        std::process::Command::new("git")
            .args(&["rev-parse", "--short=12", "HEAD"])
            .output()
            .ok()
            .and_then(|data| {
                std::str::from_utf8(&data.stdout)
                    .ok()
                    .map(|s| s.trim())
                    .map(ToString::to_string)
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| panic!("no git revision"))
    )
}
