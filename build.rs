fn main() -> anyhow::Result<()> {
    // 如果目录不存在则创建
    std::fs::create_dir_all("src/pb")?;
    let build = tonic_prost_build::configure();
    let _ = build
        .out_dir("src/pb")
        .compile_protos(&["proto/player.proto"], &["proto"]);
    Ok(())
}
