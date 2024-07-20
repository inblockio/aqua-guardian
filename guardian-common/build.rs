// use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // const PROTOS_DIR: &str = "proto";
    // const RUST_CUSTOM_TYPES_PREFIX: &str = "crate::custom_types::";

    // let custom_impls: std::collections::HashMap<&str, &'static [&str]> = std::collections::HashMap::from_iter([
    //     ("hash", &["Hash"][..]),
    //     ("signature", &["Signature"][..]),
    //     ("public_key", &["PublicKey"][..]),
    // ]);

    // let protos: Vec<_> = std::fs::read_dir(PROTOS_DIR)
    //     .unwrap()
    //     .flatten()
    //     .filter(|entry| {
    //         entry
    //             .file_name()
    //             .to_string_lossy()
    //             .strip_suffix(".proto")
    //             .map(|stem| !custom_impls.contains_key(&stem))
    //             .unwrap_or(false)
    //     })
    //     .map(|e| e.path())
    //     .collect();
    // dbg!(&protos);
    // let mut builder = tonic_build::configure()
    //     .out_dir("src/proto")
    //     .include_file("mod.rs")
    //     .emit_rerun_if_changed(true);
    // for (path, types) in custom_impls {
    //     let path = format!(".{path}");
    //     let rust_path = format!("{RUST_CUSTOM_TYPES_PREFIX}{}", path.split('.').skip(1).collect::<Vec<_>>().join("::"));
    //     builder = builder.extern_path(&path, &rust_path);
    //     for ty in types {
    //         builder = builder.extern_path(format!("{path}.{ty}"), format!("{rust_path}::{ty}"));
    //     }
    // }

    // builder.compile(&protos, &[PROTOS_DIR])?;

    Ok(())
}
