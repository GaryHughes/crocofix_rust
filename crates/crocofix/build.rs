use std::env;
use std::path::Path;
use std::process::Command;
use std::io;
use std::io::Write;

fn main()
{
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let program = format!("{}/../../lexicographer/lexicographer.py", manifest_dir.to_str().unwrap());
    let output = Path::new(&out_dir).join("FIX_4_2.rs");
    let orchestration = format!("{}/../../orchestrations/fix_repository_4_2.xml",  manifest_dir.to_str().unwrap()); 

    println!("OUT_DIR {}", out_dir.to_str().unwrap());
    println!("MANIFEST_DIR {}", manifest_dir.to_str().unwrap());
    println!("PROGRAM {}", program);
    println!("OUTPUT {}", output.to_str().unwrap());
    println!("ORCHESTRATION {}", orchestration);
    
    let output = Command::new(program)
        .arg("--output")
        .arg(output)
        .arg("--module")
        .arg("FIX_4_2")
        .arg("--orchestration")
        .arg(orchestration)
        .output()
        .expect("failed to execute process");

    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    println!("cargo::rerun-if-changed=FIX_4_2.rs");
}