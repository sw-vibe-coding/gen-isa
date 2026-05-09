use gen_isa::Spec;

fn main() {
    for path in &[
        "docs/spec-examples/cor24.toml",
        "docs/spec-examples/ibm1130.toml",
    ] {
        match Spec::parse_file(std::path::Path::new(path)) {
            Ok(s) => println!("OK {path}: {s}"),
            Err(e) => {
                eprintln!("FAIL {path}: {e}");
                std::process::exit(1);
            }
        }
    }
}
