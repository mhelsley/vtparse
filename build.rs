use std::env;
use std::path::PathBuf;

fn generate(libdir_path: &PathBuf) {
    if !std::process::Command::new("ruby")
        .arg(
            libdir_path
                .join("vtparse_gen_c_tables.rb")
                .to_str()
                .unwrap(),
        )
        .output()
        .expect("Could not find ruby")
        .status
        .success()
    {
        panic!("ruby could not generate vtparse tables");
    }
}

fn compile(sources: Vec<String>) {
    if !std::process::Command::new("cc")
        .arg("-Wall")
        .arg("-c")
        .args(sources)
        .output()
        .expect("Could not compile with \"cc\"")
        .status
        .success()
    {
        panic!("cc could not compile");
    }
}

fn archive(lib_path: &PathBuf, objs: Vec<String>) {
    if !std::process::Command::new("ar")
        .arg("rcs")
        .arg(lib_path.to_str().unwrap())
        .args(objs)
        .output()
        .expect("Could not archive with \"ar\"")
        .status
        .success()
    {
        panic!("ar could not create {}", lib_path.to_str().unwrap());
    }
    if !std::process::Command::new("ranlib")
        .arg(lib_path)
        .output()
        .expect("Could not run \"ranlib\"")
        .status
        .success()
    {
        panic!("ranlib failed");
    }
}

fn rustify(dest: &PathBuf) {
    let dest_str = dest.to_str().unwrap();
    if !std::process::Command::new("sed")
        .arg("-i")
        .arg("-e")
        .arg(r#"/^\s\+VTPARSE_\(ACTION\|STATE\)_.*$/{s/^\(\s\+\)VTPARSE_[^_]\+_/\1_/;s/_\([A-Z]\)\([A-Z]\+\)/\1\L\2\E/g;s/Csi/CSI/;s/Osc/OSC/;s/Dcs/DCS/}"#)
        //.arg("-e")
        //.arg(r#"s/unsafe extern "C" fn/unsafe extern "C-unwind" fn/"#)
        .arg(dest_str)
        .status()
        .expect("Could not run sed")
        .success() {
            panic!("Could not rustify {}", dest_str);
    }
}

fn main() {
    let libdir_path = PathBuf::from("./")
        .canonicalize()
        .expect("Could not canonicalize path");

    macro_rules! paths {
        [ $( $s:literal ),* $(,)? ] => {
            vec![ $( $s ),* ]
                .iter()
                .map(|s| String::from(libdir_path.join(*s).to_str().unwrap()))
                .collect::<Vec<String>>()
        }
    }

    let headers = paths!["wrapper.h", "vtparse.h", "vtparse_table.h",];

    let sources = paths!["vtparse.c", "vtparse_table.c",];

    let objs = sources
        .iter()
        .map(|s| s.replace(".c", ".o"))
        .collect::<Vec<String>>();

    let generator = paths!["vtparse_tables.rb", "vtparse_gen_c_tables.rb",];

    for file in headers.iter().chain(sources.iter().chain(generator.iter())) {
        println!("cargo:rerun-if-changed={}", file);
    }

    let lib_path = libdir_path.join("libvtparse.a");

    generate(&libdir_path);
    compile(sources);
    archive(&lib_path, objs);

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", libdir_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=vtparse");

    let bindings = bindgen::Builder::default()
        .blocklist_item("(STATE|ACTION)_NAMES")
        .blocklist_item("STATE_TABLE")
        .blocklist_item("(ENTRY|EXIT)_ACTIONS")
        .blocklist_item("_VTPARSE_.+_H_")
        .blocklist_item("state_change_t")
        .header("wrapper.h")
        .prepend_enum_name(false)
        .translate_enum_integer_types(true)
        .rustified_enum("vtparse_state_t")
        .rustified_enum("vtparse_action_t")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate vtparse bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let dest = out_path.join("vtparse_bindings.rs");
    bindings
        .write_to_file(&dest)
        .expect("Couldn't write vtparse_bindings!");

    rustify(&dest);
}
