use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

fn build_test_helper() {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());

    println!("cargo:warning=Building test_helper binary...");

    let target = env::var("TARGET").unwrap_or_else(|_| {
        "i686-pc-windows-msvc".to_string()
    });

    let test_helper_src = "src/bin/test_helper.rs";
    let output_name = "test_helper.exe";

    let output_path = Path::new(&target_dir)
        .join(&target)
        .join(&profile)
        .join(output_name);

    // Compile the test helper
    let status = Command::new("rustc")
        .args(&[
            test_helper_src,
            "-o", output_path.to_str().unwrap(),
        ])
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => {
            println!("cargo:warning=Successfully built test_helper at: {}", output_path.display());
        },
        Ok(_) => {
            println!("cargo:warning=Failed to compile test_helper");
        },
        Err(e) => {
            println!("cargo:warning=Error running rustc for test_helper: {}", e);
        }
    }
}

fn main() {
    // Get the package version from Cargo.toml through environment variables, and its numeric
    // components without a pre-release suffix such as -unbroken.1, which the resource compiler can't
    // take
    let version = env::var("CARGO_PKG_VERSION").expect("Failed to get package version");
    let major = env::var("CARGO_PKG_VERSION_MAJOR").expect("Failed to get package version");
    let minor = env::var("CARGO_PKG_VERSION_MINOR").expect("Failed to get package version");
    let patch = env::var("CARGO_PKG_VERSION_PATCH").expect("Failed to get package version");
    
    // Get the current git commit hash
    let commit_hash = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .map(|output| {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if hash.len() >= 10 {
                hash[..10].to_string()
            } else {
                hash
            }
        })
        .unwrap_or_else(|_| "unknown".to_string());
    //
    // Read the resource template
    let mut template_content = String::new();
    fs::File::open("resources/resources.rc.template")
        .expect("Failed to open resources.rc.template")
        .read_to_string(&mut template_content)
        .expect("Failed to read resources.rc.template");
    
    // Replace version placeholders
    let updated_content = template_content
        .replace("{{VERSION_MAJOR}}", &major)
        .replace("{{VERSION_MINOR}}", &minor)
        .replace("{{VERSION_PATCH}}", &patch)
        .replace("{{VERSION_STRING}}", &version)
        .replace("{{COMMIT}}", &commit_hash);
    
    // Write the updated content to resources.rc
    fs::write("resources/resources.rc", updated_content)
        .expect("Failed to write resources.rc");
    
    // Continue with resource compilation
    let out_dir = env::var("OUT_DIR").expect("Missing out directory?");
    let resources = Path::new(&out_dir).join("resources.lib");

    let ecode = Command::new(".\\tools\\rc.exe")
        .arg("/r")
        .arg("/fo")
        .arg(resources.as_os_str())
        .arg(".\\resources\\resources.rc")
        .spawn()
        .expect("Failed to spawn resource compiler")
        .wait()
        .expect("Failed to wait on resource compiler");

    assert!(ecode.success(), "Resource compiler failed");

    println!(
        "cargo:rustc-link-search=native={}",
        resources.parent().unwrap().to_str().unwrap()
    );
    println!(
        "cargo:rustc-link-lib={}",
        resources.file_stem().unwrap().to_str().unwrap()
    );

    // Build test_helper binary for tests
    build_test_helper();

    // Make sure we rerun the build script when resources, cargo.toml, or git head changes
    println!("cargo:rerun-if-changed=resources/resources.rc.template");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=src/bin/test_helper.rs");
}
