use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

fn main() {
    gen_gresource("res/fest.gresource.xml");
    gen_gresource("res/icons/hicolor/icons.gresource.xml");
}

fn gen_gresource(xml_path: &str) {
    println!("cargo:rerun-if-changed={}", xml_path);
    let xml_path: &Path = xml_path.as_ref();
    let xml_dir = xml_path.parent().unwrap();
    let xml_file = xml_path.file_name().unwrap();

    let mut gen_deps_process = Command::new("glib-compile-resources")
        .current_dir(xml_dir)
        .arg("--generate-dependencies")
        .arg(xml_file)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn glib-compile-resources process!");

    for line in BufReader::new(gen_deps_process.stdout.as_mut().unwrap()).lines() {
        println!(
            "cargo:rerun-if-changed={}",
            xml_dir.join(&line.unwrap()).to_str().unwrap()
        );
    }

    check_process_result(gen_deps_process.wait());
    check_process_result(
        Command::new("glib-compile-resources")
            .current_dir(xml_dir)
            .arg(xml_file)
            .status(),
    );
}

fn check_process_result(result: io::Result<ExitStatus>) {
    match result.map(|exit_status| exit_status.code()) {
        Ok(Some(0)) => {}
        Ok(Some(exit_code)) => panic!(
            "glib-compile-resources exited unsuccessfully (exit code {})!",
            exit_code
        ),
        Ok(None) => panic!("glib-compile-resources was terminated!"),
        Err(e) => panic!("glib-compile-resources failed: {}", e),
    }
}
