//! `build.rs` for `lcc-proto` — invokes `buf generate` if available, falling
//! back to pre-generated stubs when `buf` is not installed.
//!
//! To regenerate stubs: `buf generate proto` (or `just proto-gen`).

use std::process::Command;

fn main() {
    // 1. Tell cargo when to re-run this build.rs.
    println!("cargo:rerun-if-changed=../../proto");
    println!("cargo:rerun-if-changed=../../proto/buf.gen.yaml");
    println!("cargo:rerun-if-changed=../../proto/buf.lock");
    println!("cargo:rerun-if-changed=../../proto/lcc");

    // 2. Try to invoke `buf generate` if buf is available. If not, fall back
    //    to the pre-generated `src/gen/*.rs` files committed alongside the
    //    crate. This keeps `cargo build` working in environments without buf.
    let buf_available = Command::new("buf")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if buf_available {
        let status = Command::new("buf")
            .current_dir("../..")
            .args(["generate", "proto", "--template", "proto/buf.gen.yaml"])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:warning=buf generate succeeded");
            }
            Ok(s) => {
                println!("cargo:warning=buf generate exited with {s}");
            }
            Err(e) => {
                println!("cargo:warning=buf generate failed: {e}");
            }
        }
    } else {
        println!(
            "cargo:warning=buf not found; using pre-generated stubs in src/gen/"
        );
    }

    // 3. Set up prost-build fallback for `tonic::include_proto!`.
    //
    //    We don't run `tonic_build::compile_protos` here because the .proto
    //    files live outside this crate. Instead we rely on the hand-written
    //    `src/lib.rs` that uses `tonic::include_proto!` against pre-generated
    //    stubs in `src/gen/`.
    let proto_files = [
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
    for f in &proto_files {
        println!("cargo:rerun-if-changed={f}");
    }
}
