use std::process::Command;

fn main() {
    let gen_gresource = |file_prefix, dir| {
        let fname = [file_prefix, "gresource.xml"].join(".");
        let emsg = format!("Compiling {}/{}.gresource.xml failed.", dir, fname);
        Command::new("glib-compile-resources")
            .arg(fname)
            .current_dir(dir)
            .status()
            .expect(&emsg);
    };

    gen_gresource("fest", "res");
    gen_gresource("icons", "res/icons/hicolor");
}

