use std::process::Command;

fn main() {
    let gen_gresource = |file_prefix, dir| {
        let fname = [file_prefix, "gresource.xml"].join(".");
        Command::new("glib-compile-resources")
            .arg(&fname)
            .current_dir(dir)
            .status()
            .unwrap_or_else(|_| {
                panic!("Compiling {}/{}.gresource.xml failed.", dir, fname)
            });
    };

    gen_gresource("fest", "res");
    gen_gresource("icons", "res/icons/hicolor");
}
