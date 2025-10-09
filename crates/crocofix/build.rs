use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::io;
use std::io::Write;

fn main()
{
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
        .unwrap()
        .into_string()
        .expect("Failed to convert CARGO_MANIFEST_DIR into a String");
  
    let lexicographer: PathBuf = [manifest_dir.clone(), "..".into(), "..".into(), "lexicographer".into(), "lexicographer.py".into()].iter().collect();

    let fix_4_2_orchestration = [manifest_dir.clone(), "..".into(), "..".into(), "orchestrations".into(), "fix_repository_4_2.xml".into()].iter().collect();
    let fix_4_4_orchestration = [manifest_dir.clone(), "..".into(), "..".into(), "orchestrations".into(), "fix_repository_4_4.xml".into()].iter().collect();
    let fix_5_0_orchestration = [manifest_dir.clone(), "..".into(), "..".into(), "orchestrations".into(), "fix_repository_5_0SP2_EP258.xml".into()].iter().collect();

    generate_orchestration_types(&lexicographer, fix_4_2_orchestration, "FIX_4_2");
    generate_orchestration_types(&lexicographer, fix_4_4_orchestration, "FIX_4_4");
    generate_orchestration_types(&lexicographer, fix_5_0_orchestration, "FIX_5_0SP2");
}

fn generate_orchestration_types(program: &PathBuf, orchestration: PathBuf, module: &str) {

    // TODO - Make this more selective so changing one orchestration doesn't regenerate all the files.
    println!("cargo::rerun-if-changed={}", orchestration.to_str().unwrap());

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let filename = format!("{module}.rs");
    let output = Path::new(&out_dir).join(&filename);

    let output = Command::new(program)
        .arg("--output")
        .arg(output)
        .arg("--module")
        .arg(module)
        .arg("--orchestration")
        .arg(orchestration)
        .output()
        .expect("failed to execute process");

    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}