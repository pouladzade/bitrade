// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     println!("compiling p2p protos file");
//     tonic_build::configure().compile(&["./src/grpc/proto/spot.proto"], &["src/api/grpc/proto"])?;
//     Ok(())
// }
fn main() {
    tonic_build::compile_protos("src/grpc/proto/spot.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}