use std::sync::Arc;

use bili_player::{
    logger::init_logger,
    pb::{
        AddPlaylistRequest, AddPlaylistResponse, DeletedRequest, DeletedResponse, GetStateRequest,
        GetStateResponse, NextRequest, NextResponse, PauseRequest, PauseResponse, PlayBvidRequest,
        PlayBvidResponse, PlayRequest, PlayResponse, PreviousRequest, PreviousResponse,
        SetModelRequest, SetModelResponse, SetVolumeRequest, SetVolumeResponse,
        ShowPlayListRequest, ShowPlayListResponse, StopRequest, StopResponse,
        player_service_server::{PlayerService, PlayerServiceServer},
    },
    player::{
        audio_player::AudioPlayer,
        command::{PlayMode, PlayerCommand},
        play_list::load_playlist,
    },
};
use tokio::sync::{Mutex, mpsc};
use tonic::{Request, Response, Status, transport::Server};

/// 创建一个结构体，用来实现 rpc 中的 server
// #[derive(Default)]
pub struct PlayerServer {
    pub command_sender: mpsc::Sender<PlayerCommand>,
}
impl PlayerServer {
    pub fn new(command_sender: mpsc::Sender<PlayerCommand>) -> Self {
        Self { command_sender }
    }
}
/// 实现 PlayerService trait
#[tonic::async_trait]
impl PlayerService for PlayerServer {
    async fn play(&self, _request: Request<PlayRequest>) -> Result<Response<PlayResponse>, Status> {
        let result = PlayResponse {
            success: true,
            message: "音乐正在播放中".into(),
        };
        let _res = self.command_sender.send(PlayerCommand::Play).await;
        Ok(Response::new(result))
    }

    async fn play_bvid(
        &self,
        request: Request<PlayBvidRequest>,
    ) -> Result<Response<PlayBvidResponse>, Status> {
        let input = request.into_inner();
        let info = format!("即将播放: {}", input.bvid);
        let _res = self
            .command_sender
            .send(PlayerCommand::PlayBvid(input))
            .await;
        let result = PlayBvidResponse {
            success: true,
            message: info,
        };
        Ok(Response::new(result))
    }

    async fn pause(
        &self,
        _request: Request<PauseRequest>,
    ) -> Result<Response<PauseResponse>, Status> {
        let _ = self.command_sender.send(PlayerCommand::Pause).await;
        let result = PauseResponse {
            success: true,
            message: "暂停播放".into(),
        };
        Ok(Response::new(result))
    }

    async fn next(&self, _request: Request<NextRequest>) -> Result<Response<NextResponse>, Status> {
        let _ = self.command_sender.send(PlayerCommand::Next).await;
        let result = NextResponse {
            success: true,
            message: "播放下一首歌曲".into(),
        };
        Ok(Response::new(result))
    }

    async fn previous(
        &self,
        _request: Request<PreviousRequest>,
    ) -> Result<Response<PreviousResponse>, Status> {
        let _ = self.command_sender.send(PlayerCommand::Previous).await;
        let result = PreviousResponse {
            success: true,
            message: "播放上一首歌曲".into(),
        };
        Ok(Response::new(result))
    }

    async fn stop(&self, _request: Request<StopRequest>) -> Result<Response<StopResponse>, Status> {
        let _ = self.command_sender.send(PlayerCommand::Stop).await;
        let result = StopResponse {
            success: true,
            message: "停止播放".into(),
        };
        Ok(Response::new(result))
    }

    async fn set_model(
        &self,
        request: Request<SetModelRequest>,
    ) -> Result<Response<SetModelResponse>, Status> {
        let input = request.into_inner();
        let model = PlayMode::from_string(input.model.as_str()).unwrap();
        if (self
            .command_sender
            .send(PlayerCommand::SetModel(input))
            .await)
            .is_ok()
        {
            let info = format!("{} 模式设置成功!", model.get_string());
            let result = SetModelResponse {
                success: true,
                message: info,
            };
            return Ok(Response::new(result));
        } else {
            Err(Status::internal("设置播放模式失败"))
        }
    }
    async fn add_playlist(
        &self,
        _request: Request<AddPlaylistRequest>,
    ) -> Result<Response<AddPlaylistResponse>, Status> {
        todo!()
    }
    async fn deleted(
        &self,
        _request: Request<DeletedRequest>,
    ) -> Result<Response<DeletedResponse>, Status> {
        todo!()
    }
    async fn get_state(
        &self,
        _request: Request<GetStateRequest>,
    ) -> Result<Response<GetStateResponse>, Status> {
        todo!()
    }
    async fn show_play_list(
        &self,
        _request: Request<ShowPlayListRequest>,
    ) -> Result<Response<ShowPlayListResponse>, Status> {
        todo!()
    }
    async fn set_volume(
        &self,
        _request: Request<SetVolumeRequest>,
    ) -> Result<Response<SetVolumeResponse>, Status> {
        todo!()
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    init_logger("info").await?;
    // 定义播放模式为列表循环播放
    let play_mode = PlayMode::Normal;
    // 定义初始播放索引为0
    let initial_track_index = 0;
    // 加载播放列表
    load_playlist().await?;
    // 创建播放命令发送和接收的通道
    let (player_command_send, player_command_recv) = mpsc::channel::<PlayerCommand>(1);
    // 创建播放服务
    let audio_player = AudioPlayer::new(
        play_mode,
        initial_track_index,
        Arc::new(Mutex::new(player_command_recv)),
    )
    .await?;
    // 启动播放服务
    tokio::task::spawn({
        let audio_player = audio_player.clone();
        async move {
            audio_player.play_playlist().await.unwrap();
        }
    });
    // grpc 服务地址
    let addr = "[::1]:50052".parse().unwrap();
    // 创建grpc服务
    let svc = PlayerServer::new(player_command_send);
    tracing::info!("UserServiceServer listening on {addr}");
    // 启动服务
    Server::builder()
        .add_service(PlayerServiceServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
