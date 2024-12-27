pub mod vad_grpc_v1 {
    include!("vad_grpc.v1.rs");

    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("server_descriptor");
}
