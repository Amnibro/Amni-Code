// Embeds everything under static/vendor/ (Monaco editor, etc.) so the UI works with no network.
use std::{env, fs, path::Path};
fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            walk(&p, root, out);
        } else {
            let rel = p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            out.push((rel, fs::canonicalize(&p).unwrap().to_string_lossy().replace('\\', "/")));
        }
    }
}
fn main() {
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("static").join("vendor");
    println!("cargo:rerun-if-changed=static/vendor");
    let mut files = Vec::new();
    walk(&root, &root, &mut files);
    let mut code = String::from("pub fn vendor_asset(path: &str) -> Option<&'static [u8]> {\n    match path {\n");
    for (rel, abs) in &files {
        code.push_str(&format!("        {:?} => Some(include_bytes!({:?})),\n", rel, abs));
    }
    code.push_str("        _ => None,\n    }\n}\n");
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("vendor_assets.rs"), code).unwrap();
}
