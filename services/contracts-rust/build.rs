fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    std::env::set_var("PROTOC", protoc);
    let root = "../../contracts/proto";
    let files = [
        "../../contracts/proto/tektalk/v1/common.proto",
        "../../contracts/proto/tektalk/v1/identity.proto",
        "../../contracts/proto/tektalk/v1/messaging.proto",
        "../../contracts/proto/tektalk/v1/sync.proto",
        "../../contracts/proto/tektalk/v1/plugin_registry.proto",
        "../../contracts/proto/tektalk/v1/account.proto",
        "../../contracts/proto/tektalk/v1/chat.proto",
        "../../contracts/proto/tektalk/v1/session_management.proto",
        "../../contracts/proto/tektalk/v1/consent_management.proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&files, &[root])?;
    for file in files {
        println!("cargo:rerun-if-changed={file}");
    }
    Ok(())
}
