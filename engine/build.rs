fn main() {
    tonic_build::compile_protos("src/grpc/proto/spot.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
