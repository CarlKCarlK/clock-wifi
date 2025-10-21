//! This build script requests that `cargo` re-build the crate whenever `memory.x` is changed.
//! `memory.x`is a linker script--a text file telling the final step of the compilation process
//! how modules and program sections (parts of the program) should be located in memory when loaded
//! on hardware.
//! Linker scripts like `memory.x` are not normally a part of the build process and changes to it
//! would ordinarily be ignored by the build process.

use std::{env, fs::File, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    // Put `memory.x` in our output directory and ensure it's on the linker search path.
    let out =
        &PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR environment variable is not set"));
    File::create(out.join("memory.x"))?.write_all(include_bytes!("memory.x"))?;
    println!("cargo:rustc-link-search={}", out.display());

    // Tell `cargo` to rebuild project if `memory.x` linker script file changes
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rerun-if-changed=build.rs"); // Re-run if this file changes
    println!("cargo:rerun-if-changed=*"); // Re-run if any file in the project changes

    // WiFi credentials and timezone configuration
    // 1) Try project-local .env (ignored by git)
    let _ = dotenvy::from_filename(".env");

    // 2) Fall back to HOME/.pico.env (Windows: USERPROFILE)
    if env::var("WIFI_SSID").is_err() || env::var("WIFI_PASS").is_err() || env::var("UTC_OFFSET_MINUTES").is_err() {
        let home = env::var_os("USERPROFILE").or_else(|| env::var_os("HOME"))
            .expect("Could not determine home directory (USERPROFILE/HOME not set)");
        let mut p = PathBuf::from(home);
        p.push(".pico.env");
        let _ = dotenvy::from_path(&p);
    }

    // 3) Require all vars (fail fast with clear message)
    let ssid = env::var("WIFI_SSID")
        .expect("Missing WIFI_SSID (set in ./.env or ~/.pico.env)");
    let pass = env::var("WIFI_PASS")
        .expect("Missing WIFI_PASS (set in ./.env or ~/.pico.env)");
    let utc_offset = env::var("UTC_OFFSET_MINUTES")
        .expect("Missing UTC_OFFSET_MINUTES (set in ./.env or ~/.pico.env, e.g., -420 for PST)");

    // 4) Expose as compile-time constants
    println!("cargo:rustc-env=WIFI_SSID={ssid}");
    println!("cargo:rustc-env=WIFI_PASS={pass}");
    println!("cargo:rustc-env=UTC_OFFSET_MINUTES={utc_offset}");

    // Optional: don't rebuild unless these change
    println!("cargo:rerun-if-env-changed=WIFI_SSID");
    println!("cargo:rerun-if-env-changed=WIFI_PASS");
    println!("cargo:rerun-if-env-changed=UTC_OFFSET_MINUTES");
    println!("cargo:rerun-if-env-changed=DST_OFFSET_MINUTES");
    println!("cargo:rerun-if-changed=.env");

    Ok(())
}
