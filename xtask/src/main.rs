use std::process::{Command, ExitStatus};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("build-web") => build_web(),
        Some("codegen") => codegen(),
        Some("build") => {
            build_web()?;
            codegen()?;
            cargo_build()
        }
        _ => {
            eprintln!(
                "Usage: cargo xtask <task>\n\
                 Tasks:\n  \
                   build-web  Build web/dist with Vite\n  \
                   codegen    Export TypeScript types from toolbelt-types\n  \
                   build      build-web + codegen + cargo build"
            );
            std::process::exit(1);
        }
    }
}

fn build_web() -> Result<()> {
    println!("==> Building web/dist");
    let root = project_root();
    let web = root.join("web");
    run(Command::new("npm").arg("install").current_dir(&web))?;
    run(Command::new("npm").args(["run", "build"]).current_dir(&web))?;
    Ok(())
}

fn codegen() -> Result<()> {
    println!("==> Generating TypeScript types");
    run(
        Command::new("cargo")
            .args(["test", "--package", "toolbelt-types", "--", "export_bindings"])
            .current_dir(project_root()),
    )?;
    Ok(())
}

fn cargo_build() -> Result<()> {
    println!("==> cargo build");
    run(Command::new("cargo").args(["build"]).current_dir(project_root()))?;
    Ok(())
}

fn run(cmd: &mut Command) -> Result<ExitStatus> {
    println!("   $ {:?}", cmd);
    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("command failed: {:?}", cmd).into());
    }
    Ok(status)
}

fn project_root() -> std::path::PathBuf {
    // xtask lives at <root>/xtask — go up one level
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(manifest).parent().unwrap().to_path_buf()
}
