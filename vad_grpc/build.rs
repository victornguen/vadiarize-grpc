use protox::prost::{bytes::BytesMut, Message as _};
use std::env;
use std::{
    fs::File,
    io::{BufWriter, Write as _},
    path::{Path, PathBuf},
};

use tonic_build::FileDescriptorSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let proto_files = ["./proto/vad_grpc.proto"];
    let include_dirs = ["proto"];

    let file_descriptor_set_path = out_dir.join("inference_server_descriptor.bin");

    let fds = protox::compile(&proto_files, &include_dirs).unwrap();
    write_fds(&fds, &file_descriptor_set_path);

    let proto_gen_dir = "./src/pb";

    std::fs::create_dir_all(proto_gen_dir)?;

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_server(true)
        .build_client(true)
        .out_dir(proto_gen_dir)
        .compile_fds(fds)
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));

    Ok(())
}

fn write_fds(fds: &FileDescriptorSet, path: &Path) {
    let mut writer = BufWriter::new(File::create(path).unwrap());
    let mut buf = BytesMut::with_capacity(fds.encoded_len());
    fds.encode(&mut buf).unwrap();
    writer.write_all(&buf).unwrap();
}
