fn main() {
    println!("cargo:rerun-if-env-changed=COSMOSTRIX_BUILD");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");

    let build_id = if let Ok(v) = std::env::var("COSMOSTRIX_BUILD") {
        if !v.is_empty() {
            v
        } else {
            infer_build_id()
        }
    } else {
        infer_build_id()
    };

    println!("cargo:rustc-env=COSMOSTRIX_BUILD={}", build_id);

    let sha = git_short_sha()
        .or_else(|| env_short_sha("GITHUB_SHA"))
        .unwrap_or_default();
    println!("cargo:rustc-env=COSMOSTRIX_GIT_SHA={}", sha);

    // Embed rustc version
    let rustc_version = detect_rustc_version();
    println!("cargo:rustc-env=COSMOSTRIX_RUSTC_VERSION={}", rustc_version);

    // Embed LTO setting from profile env vars
    let lto = detect_lto();
    println!("cargo:rustc-env=COSMOSTRIX_LTO={}", lto);

    // Embed panic strategy from profile env vars
    let panic = detect_panic();
    println!("cargo:rustc-env=COSMOSTRIX_PANIC={}", panic);

    // Embed strip setting from profile env vars
    let strip = detect_strip();
    println!("cargo:rustc-env=COSMOSTRIX_STRIP={}", strip);
}

fn env_short_sha(name: &str) -> Option<String> {
    let v = std::env::var(name).ok()?;
    let v = v.trim();
    if v.is_empty() {
        return None;
    }
    let n = v.len().min(7);
    let short = &v[..n];
    if short.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(short.to_ascii_lowercase())
    } else {
        None
    }
}

fn git_short_sha() -> Option<String> {
    use std::process::Command;

    let out = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if s.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(s.to_ascii_lowercase())
    } else {
        None
    }
}

fn infer_build_id() -> String {
    let os_raw = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    let os = match os_raw.as_str() {
        "macos" => "darwin",
        other => other,
    };

    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string());
    let features = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();

    if arch == "x86_64" {
        if os == "linux" {
            let variant = if features.contains("avx512f") {
                "v4"
            } else if features.contains("avx2") {
                "v3"
            } else if features.contains("sse4.2") || features.contains("sse4_2") {
                "v2"
            } else {
                "v1"
            };
            format!("{os}-{arch}-{variant}")
        } else {
            format!("{os}-{arch}")
        }
    } else {
        format!("{os}-{arch}-native")
    }
}

fn detect_rustc_version() -> String {
    use std::process::Command;

    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Detect LTO setting from the active Cargo profile.
///
/// Checks `CARGO_PROFILE_<UPPERCASE_PROFILE>_LTO` env vars for the
/// current build profile. The profile name is available via
/// `CARGO_PROFILE_NAME` (set by Cargo since 1.63).
fn detect_lto() -> String {
    let profile = std::env::var("CARGO_PROFILE_NAME").unwrap_or_else(|_| "release".to_string());
    let key = format!("CARGO_PROFILE_{}_LTO", profile.to_ascii_uppercase());
    match std::env::var(&key).as_deref() {
        Ok("fat") => "fat",
        Ok("thin") => "thin",
        Ok("off") | Ok("false") | Ok("n") => "off",
        Ok(v) => v,
        Err(_) => "off",
    }
    .to_string()
}

/// Detect panic strategy from the active Cargo profile.
fn detect_panic() -> String {
    let profile = std::env::var("CARGO_PROFILE_NAME").unwrap_or_else(|_| "release".to_string());
    let key = format!("CARGO_PROFILE_{}_PANIC", profile.to_ascii_uppercase());
    match std::env::var(&key).as_deref() {
        Ok("abort") => "abort",
        Ok("unwind") => "unwind",
        Ok(v) => v,
        Err(_) => "unwind",
    }
    .to_string()
}

/// Detect strip setting from the active Cargo profile.
fn detect_strip() -> String {
    let profile = std::env::var("CARGO_PROFILE_NAME").unwrap_or_else(|_| "release".to_string());
    let key = format!("CARGO_PROFILE_{}_STRIP", profile.to_ascii_uppercase());
    match std::env::var(&key).as_deref() {
        Ok("true") | Ok("symbols") => "yes",
        Ok("false") | Ok("none") => "no",
        Ok("debuginfo") => "debuginfo",
        Ok(v) => v,
        Err(_) => "no",
    }
    .to_string()
}
