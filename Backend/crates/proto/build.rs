//! `build.rs` for `lcc-proto` — generates the protobuf/tonic stubs that the
//! `proto-binary` feature compiles via `include!("gen/…")`.
//!
//! The default build needs none of this (the crate ships hand-written
//! serde types), so codegen only runs when `proto-binary` is enabled.
//!
//! Generation order:
//!   1. `buf generate`, when `buf` is on PATH (matches `proto/buf.gen.yaml`,
//!      and is what `just proto-gen` uses).
//!   2. Otherwise `tonic-build` driven by `protoc`, writing the same
//!      `src/gen/lcc.v1.*.rs` layout the `include!`s expect.
//!
//! Without step 2 the feature was unbuildable: `buf` is not present in CI or
//! in the audit sandbox, `src/gen/` held only a `.gitkeep`, and both
//! `just rust-test` and the Rust CI job run `cargo test --workspace
//! --all-features`, so `--all-features` enabled `proto-binary` and the build
//! died on `couldn't find file crates/proto/src/gen/lcc.v1.common.trace.rs`.

use std::path::{Path, PathBuf};
use std::process::Command;

const PROTOS: &[&str] = &[
    "../../proto/lcc/v1/common/trace.proto",
    "../../proto/lcc/v1/common/pagination.proto",
    "../../proto/lcc/v1/compliance/governor.proto",
    "../../proto/lcc/v1/compliance/admin.proto",
    "../../proto/lcc/v1/integration/execute.proto",
    "../../proto/lcc/v1/intelligence/ai.proto",
    "../../proto/lcc/v1/intelligence/opportunity.proto",
    "../../proto/lcc/v1/intelligence/kb.proto",
    "../../proto/lcc/v1/intelligence/voice.proto",
    "../../proto/lcc/v1/intelligence/scoring.proto",
    "../../proto/lcc/v1/profile/profile.proto",
    "../../proto/lcc/v1/content/content.proto",
    "../../proto/lcc/v1/engagement/engagement.proto",
    "../../proto/lcc/v1/network/network.proto",
    "../../proto/lcc/v1/outreach/outreach.proto",
    "../../proto/lcc/v1/analytics/analytics.proto",
    "../../proto/lcc/v1/approval/approval.proto",
    "../../proto/lcc/v1/identity/identity.proto",
    "../../proto/lcc/v1/events/events.proto",
];

fn main() {
    for f in PROTOS {
        println!("cargo:rerun-if-changed={f}");
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../proto/buf.gen.yaml");

    let want_binary = std::env::var_os("CARGO_FEATURE_PROTO_BINARY").is_some();
    if !want_binary {
        return;
    }

    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()),
    );
    let out_dir = manifest_dir.join("src").join("gen");

    if buf_available() {
        let status = Command::new("buf")
            .current_dir(manifest_dir.join("../.."))
            .args(["generate", "proto", "--template", "proto/buf.gen.yaml"])
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("cargo:warning=buf generate succeeded");
                copy_buf_output(&manifest_dir, &out_dir);
                return;
            }
            Ok(s) => println!("cargo:warning=buf generate exited with {s}"),
            Err(e) => println!("cargo:warning=buf generate failed: {e}"),
        }
    }

    generate_with_protoc(&manifest_dir, &out_dir);
}

fn buf_available() -> bool {
    Command::new("buf")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `buf.gen.yaml` emits into `proto/gen/rust/…`; mirror it into `src/gen/`
/// so the `include!("gen/…")` paths in `src/lib.rs` resolve.
fn copy_buf_output(manifest_dir: &Path, out_dir: &Path) {
    let buf_out = manifest_dir.join("../../proto/gen/rust");
    if !buf_out.is_dir() {
        return;
    }
    if let Err(e) = copy_tree(&buf_out, out_dir) {
        println!("cargo:warning=copying buf output failed: {e}");
    }
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Drive `protoc` through `tonic-build`, writing `<package>.<file>.rs` into
/// `src/gen/` — the exact layout `prost` uses by default and the one
/// `src/lib.rs` includes.
fn generate_with_protoc(manifest_dir: &Path, out_dir: &Path) {
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        println!("cargo:warning=could not create {}: {e}", out_dir.display());
        return;
    }

    let proto_root = manifest_dir.join("../../proto");
    let mut includes = vec![proto_root.clone()];
    // google/protobuf well-known types are imported by the .proto files.
    includes.push(PathBuf::from("/usr/include"));

    let files: Vec<PathBuf> = PROTOS
        .iter()
        .map(|f| manifest_dir.join(f))
        .filter(|p| p.exists())
        .collect();

    if files.is_empty() {
        println!("cargo:warning=no .proto sources found; skipping codegen");
        return;
    }

    if let Err(e) = tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir(out_dir)
        // `google.type.Date` is NOT in `prost_types` (that crate only carries
        // the `google.protobuf` well-known types), so it has to be generated
        // in-tree. Mapping it to `::prost_types` failed with
        // "cannot find type `Date` in crate `::prost_types`".
        //
        // It is emitted into its own top-level `google` module in lib.rs, and
        // prost's generated reference from `lcc.v1.network` resolves to it
        // because both are siblings at the crate root.
        .compile_protos(&files, &includes)
    {
        println!("cargo:warning=protoc codegen failed: {e}");
    }
}
