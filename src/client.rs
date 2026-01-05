use bili_player::pb::{
    NextRequest, PauseRequest, PlayBvidRequest, PlayRequest, PreviousRequest, SetModelRequest,
    StopRequest, player_service_client::PlayerServiceClient,
};
use clap::{Parser, Subcommand};
#[derive(Debug, Parser)]
#[command(
    name = "bpc",
    about = "Control the bilibili player.",
    version = "1.0.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "播放指定歌曲或继续播放")]
    Play(PlayCommand),

    #[command(about = "暂停播放")]
    Pause,

    #[command(about = "播放下一首歌曲")]
    Next,

    #[command(about = "播放上一首歌曲")]
    Previous,

    #[command(about = "停止 Bilibili Player")]
    Stop,

    #[command(about = "设置播放模式")]
    Mode(ModeCommand),

    #[command(about = "添加歌曲到播放列表")]
    Add(AddCommand),

    #[command(about = "在播放列表中查找歌曲")]
    Find(FindCommand),

    #[command(about = "从播放列表中删除歌曲")]
    Delete(DeleteCommand),

    #[command(about = "显示播放列表")]
    Playlist,
}

#[derive(Debug, Parser)]
struct PlayCommand {
    #[arg(short = 'b', long = "bvid", help = "要播放的 bvid")]
    bvid: Option<String>,
}

#[derive(Debug, Parser)]
struct AddCommand {
    #[arg(short = 'b', long = "bvid", help = "要导入的 bvid")]
    bvid: Option<String>,
}
#[derive(Debug, Parser)]
struct DeleteCommand {
    #[arg(short = 'b', long = "bvid", help = "按 bvid 删除")]
    bvid: String,
}
#[derive(Debug, Parser)]
struct ModeCommand {
    #[arg(short = 'n', long = "normal", action = clap::ArgAction::SetTrue, help = "设置播放模式为循环播放")]
    normal_mode: bool,
    #[arg(short = 's', long = "shuffle", action = clap::ArgAction::SetTrue, help = "设置播放模式为随机播放")]
    shuffle_mode: bool,
    #[arg(short = 'r', long = "repeat", action = clap::ArgAction::SetTrue, help = "设置播放模式为单曲循环")]
    repeat_mode: bool,
}
#[derive(Debug, Parser)]
struct FindCommand {
    #[arg(short = 'b', long = "bvid", help = "按 bvid 查找")]
    bvid: Option<String>,
    #[arg(short = 'c', long = "cid", help = "按 cid 查找")]
    cid: Option<String>,
    #[arg(short = 't', long = "title", help = "按标题查找")]
    title: Option<String>,
    #[arg(short = 'o', long = "owner", help = "按作者查找")]
    owner: Option<String>,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 解析命令
    let cli = Cli::parse();
    eprintln!("Cli:{:?}", cli);
    // 创建连接
    let mut client = PlayerServiceClient::connect("http://[::1]:50052").await?;
    match cli.command {
        // 播放，如果有传入 bvid，则播放 bvid 的歌曲，否则播放当前歌曲
        Commands::Play(play_cmd) => {
            if let Some(bvid) = play_cmd.bvid {
                let request = tonic::Request::new(PlayBvidRequest { bvid });
                let response = client.play_bvid(request).await?.into_inner();
                if response.success {
                    eprintln!("{}", response.message);
                };
            } else {
                let request = tonic::Request::new(PlayRequest {});
                let response = client.play(request).await?.into_inner();
                if response.success {
                    eprintln!("{}", response.message);
                };
            }
        }
        // 暂停播放
        Commands::Pause => {
            let request = tonic::Request::new(PauseRequest {});
            let response = client.pause(request).await?.into_inner();
            if response.success {
                eprintln!("{}", response.message);
            };
        }
        // 播放下一首
        Commands::Next => {
            let request = tonic::Request::new(NextRequest {});
            let response = client.next(request).await?.into_inner();
            if response.success {
                eprintln!("{}", response.message);
            };
        }
        // 播放上一首
        Commands::Previous => {
            let request = tonic::Request::new(PreviousRequest {});
            let response = client.previous(request).await?.into_inner();
            if response.success {
                eprintln!("{}", response.message);
            };
        }
        // 停止播放
        Commands::Stop => {
            let request = tonic::Request::new(StopRequest {});
            let response = client.stop(request).await?.into_inner();
            if response.success {
                eprintln!("{}", response.message);
            };
        }
        Commands::Mode(mode_cmd) => {
            let model = if mode_cmd.shuffle_mode {
                "shuffle".into() // 随机播放
            } else if mode_cmd.repeat_mode {
                "repeat".into() // 单曲循环播放
            } else {
                "normal".into() // 列表播放
            };
            let request = tonic::Request::new(SetModelRequest { model });
            let response = client.set_model(request).await?.into_inner();
            if response.success {
                eprintln!("{}", response.message);
            };
        }
        Commands::Add(_add_cmd) => {}
        Commands::Delete(_delete_cmd) => {}
        Commands::Find(_find_cmd) => {}
        Commands::Playlist => {}
    }
    Ok(())
}
