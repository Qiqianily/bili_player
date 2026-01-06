use crate::{
    errors::ApplicationError,
    fetch::verify::fetch_and_verify_audio_url,
    player::{
        command::{PlayMode, PlayerCommand},
        play_list::{
            PLAYLIST, get_current_music, move_to_next_music, move_to_previous_music,
            set_current_music_index,
        },
    },
};
use futures_util::StreamExt;
use gstreamer::{
    MessageView,
    prelude::{ElementExt, GstBinExt, GstBinExtManual, GstObjectExt, PadExt},
};
use gstreamer::{glib::object::ObjectExt, prelude::ElementExtManual};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::{Mutex, RwLock, mpsc};
// 用来存放播放状态
#[derive(Clone)]
pub struct AudioPlayer {
    pub pipeline: Arc<gstreamer::Pipeline>,
    pub client: Arc<reqwest::Client>,
    pub play_mode: Arc<RwLock<PlayMode>>, // 播放模式，如 "Normal", "Shuffle", "Repeat"
    pub volume: Arc<AtomicU32>,           // 使用原子整型存储音量
    pub command_receiver: Arc<Mutex<mpsc::Receiver<PlayerCommand>>>, // 命令接收器
    pub eos_sender: mpsc::Sender<()>,     // 结束信号发送器
}

impl AudioPlayer {
    pub async fn new(
        play_mode: PlayMode,
        volume: u32,
        initial_music_index: usize,
        command_receiver: Arc<Mutex<mpsc::Receiver<PlayerCommand>>>,
    ) -> Result<Self, ApplicationError> {
        // 初始化 gstreamer
        gstreamer::init().map_err(|e| ApplicationError::InitError(e.to_string()))?;
        // 初始化音频播放器
        let pipeline = Arc::new(gstreamer::Pipeline::new());
        // 创建 client
        let client = Arc::new(reqwest::Client::new());
        set_current_music_index(initial_music_index).await?;
        // 创建接收音频流结束的通道
        let (eos_sender, eos_receiver) = mpsc::channel(1);
        tracing::info!("GStreamer created successfully.");
        let audio_player = AudioPlayer {
            pipeline,
            client,
            volume: Arc::new(AtomicU32::new(volume)),
            play_mode: Arc::new(RwLock::new(play_mode)),
            command_receiver,
            eos_sender,
        };
        // 启动 EOS 监听器
        audio_player.start_eos_listener(eos_receiver).await?;
        // 返回 audio_player
        Ok(audio_player)
    }
    /// 设置音量 (0-100)
    pub fn set_volume(&self, percentage: u32) -> Result<(), ApplicationError> {
        if percentage > 200 {
            return Err(ApplicationError::VolumeError("音量值在：0-200".into()));
        }

        self.volume.store(percentage * 10, Ordering::Relaxed);

        // 应用到当前播放
        self.apply_volume_to_pipeline();

        Ok(())
    }
    /// 获取音量百分比 (0-100)
    pub fn get_volume(&self) -> u32 {
        self.volume.load(Ordering::Relaxed) / 10
    }
    /// 获取 GStreamer 音量值 (0.0-2.0)
    fn get_gstreamer_volume(&self) -> f64 {
        let volume_int = self.volume.load(Ordering::Relaxed);
        volume_int as f64 / 1000.0
    }
    /// 应用到当前pipeline
    fn apply_volume_to_pipeline(&self) {
        if let Some(volume_elem) = self.pipeline.by_name("audio_volume") {
            let vol_value = self.get_gstreamer_volume();
            volume_elem.set_property("volume", vol_value);
        }
    }
    /// 处理播放器命令
    pub async fn handle_command(&mut self, command: PlayerCommand) {
        match command {
            PlayerCommand::Play => todo!(),
            PlayerCommand::PlayBvid(_play_bvid_request) => todo!(),
            PlayerCommand::Pause => todo!(),
            PlayerCommand::Next => todo!(),
            PlayerCommand::Previous => todo!(),
            PlayerCommand::Stop => todo!(),
            PlayerCommand::SetModel(_set_model_request) => todo!(),
            PlayerCommand::SetVolume(_set_volume_request) => todo!(),
            PlayerCommand::AddPlaylist(_add_playlist_request) => todo!(),
            PlayerCommand::Delete(_deleted_request) => todo!(),
            PlayerCommand::GetState(_sender) => todo!(),
            PlayerCommand::ShowPlaylist() => todo!(),
            PlayerCommand::Seek(_seek_time) => todo!(),
        }
    }

    // 监听 EOS 事件
    async fn start_eos_listener(
        &self,
        mut eos_receiver: mpsc::Receiver<()>,
    ) -> Result<(), ApplicationError> {
        let pipeline = Arc::clone(&self.pipeline);
        let client = Arc::clone(&self.client);
        let play_mode = Arc::clone(&self.play_mode);
        let volume_value = self.get_gstreamer_volume();
        // 开启一个线程用来接收播放完成的信号
        tokio::task::spawn(async move {
            while (eos_receiver.recv().await).is_some() {
                tracing::info!("Music finished playing. Handling EOS...");
                let current_play_mode = *play_mode.read().await;
                // 处理播放完成的逻辑
                if current_play_mode != PlayMode::Repeat
                    && let Err(e) = move_to_next_music(current_play_mode).await
                {
                    tracing::error!("Failed to move to next music: {}", e);
                    continue;
                }
                if let Err(e) = play_music(&pipeline, volume_value, &client).await {
                    tracing::error!("Failed to play next music: {}", e);
                }
            }
        });

        Ok(())
    }
    /// 播放列表中的歌曲
    pub async fn play_playlist(&self) -> Result<(), ApplicationError> {
        let pipeline = Arc::clone(&self.pipeline);
        let client = Arc::clone(&self.client);
        let play_mode = Arc::clone(&self.play_mode);
        let command_receiver = Arc::clone(&self.command_receiver);
        let eos_sender = self.eos_sender.clone();
        let volume_value = self.get_gstreamer_volume();
        // Watch GStreamer bus messages
        let bus = self.pipeline.bus().ok_or_else(|| {
            ApplicationError::PipelineError("Failed to get GStreamer bus".to_string())
        })?;

        let bus_receiver = bus.stream().for_each(move |msg| {
            let eos_sender = eos_sender.clone();
            async move {
                match msg.view() {
                    MessageView::Eos(_) => {
                        tracing::info!("EOS message received, sending signal.");
                        if eos_sender.send(()).await.is_err() {
                            tracing::error!("Failed to send EOS signal");
                        }
                    }
                    MessageView::Error(err) => {
                        tracing::error!("Error from GStreamer pipeline: {}", err);
                    }
                    _ => (),
                }
            }
        });

        // Listen for commands and process GStreamer messages concurrently
        tokio::task::spawn(async move {
            let mut command_receiver = command_receiver.lock().await;
            tokio::pin!(bus_receiver);
            loop {
                tokio::select! {
                    command = command_receiver.recv() => {
                        if let Some(command) = command {
                            // self.handle_command(command).await;
                            match command {
                                PlayerCommand::Play => {
                                    tracing::info!("Resume playback");
                                    if let Err(e) = pipeline.set_state(gstreamer::State::Playing) {
                                        tracing::error!("Failed to play: {}", e);
                                    }
                                }
                                PlayerCommand::PlayBvid(play_bvid_request) => {
                                    tracing::info!("Play {}", play_bvid_request.bvid);
                                    {
                                        let playlist = PLAYLIST.lock().await;
                                        let playlist = playlist.as_ref().unwrap();

                                        if let Some(new_index) = playlist.find_music_index(&play_bvid_request.bvid).await {
                                            set_current_music_index(new_index).await.ok();
                                        } else {
                                            tracing::error!("Music with bvid {} not found in the playlist", play_bvid_request.bvid);
                                        }
                                    }
                                    if let Err(e) = play_music(&pipeline,volume_value, &client).await {
                                        tracing::error!("Failed to play track after set new bvid: {}", e);
                                    }
                                }
                                PlayerCommand::Pause => {
                                    tracing::info!("Pause");
                                    if let Err(e) = pipeline.set_state(gstreamer::State::Paused) {
                                        tracing::error!("Failed to pause: {}", e);
                                    }
                                }
                                PlayerCommand::Next => {
                                    tracing::info!("Play next song");
                                    let current_play_mode = *play_mode.read().await;
                                    let mode = if current_play_mode == PlayMode::Repeat {
                                        PlayMode::Normal
                                    } else {
                                        current_play_mode
                                    };
                                    if let Err(e) = move_to_next_music(mode).await {
                                        tracing::error!("Failed to skip to next track: {}", e);
                                    } else if let Err(e) = play_music(&pipeline,volume_value, &client).await {
                                        tracing::error!("Failed to play next track: {}", e);
                                    }
                                }
                                PlayerCommand::Previous => {
                                    tracing::info!("Play previous song");
                                    let current_play_mode = *play_mode.read().await;
                                    let mode = if current_play_mode == PlayMode::Repeat {
                                        PlayMode::Normal
                                    } else {
                                        current_play_mode
                                    };
                                    if let Err(e) = move_to_previous_music(mode).await {
                                        tracing::error!("Failed to skip to previous track: {}", e);
                                    } else if let Err(e) = play_music(&pipeline, volume_value, &client).await {
                                        tracing::error!("Failed to play previous track: {}", e);
                                    }
                                }
                                PlayerCommand::Stop => {
                                    if let Err(e) = pipeline.set_state(gstreamer::State::Null) {
                                        tracing::error!("Failed to stop: {}", e);
                                    }
                                }
                                PlayerCommand::SetModel(set_model_request) => {
                                    let mut write_guard = play_mode.write().await;
                                    *write_guard = PlayMode::from_string(set_model_request.model.as_str()).unwrap_or(PlayMode::Normal);
                                }
                                PlayerCommand::SetVolume(set_volume_request) => {
                                    pipeline.set_property("volume", set_volume_request.volume);
                                    tracing::info!("Volume set to {}", set_volume_request.volume);
                                }
                                PlayerCommand::AddPlaylist(_add_playlist_request) => todo!(),
                                PlayerCommand::Delete(_deleted_request) => todo!(),
                                PlayerCommand::GetState(_sender) => todo!(),
                                PlayerCommand::ShowPlaylist() => todo!(),
                                PlayerCommand::Seek(_seek_time) => todo!(),
                            }
                        }
                    },
                    _ = &mut bus_receiver => {},
                }
            }
        });

        play_music(&self.pipeline, volume_value, &self.client).await?;
        Ok(())
    }
}

/// 播放音乐
pub async fn play_music(
    pipeline: &gstreamer::Pipeline,
    volume_value: f64,
    client: &reqwest::Client,
) -> Result<(), ApplicationError> {
    pipeline
        .set_state(gstreamer::State::Null)
        .map_err(|_| ApplicationError::StateError("Failed to set pipeline to Null".to_string()))?;

    for element in pipeline.children() {
        pipeline.remove(&element).map_err(|_| {
            ApplicationError::ElementError("Failed to remove element from pipeline".to_string())
        })?;
    }

    pipeline
        .set_state(gstreamer::State::Ready)
        .map_err(|_| ApplicationError::StateError("Failed to set pipeline to Ready".to_string()))?;

    let music = get_current_music().await?;
    let url = fetch_and_verify_audio_url(client, &music.bvid, &music.cid).await?;

    set_pipeline_uri_with_headers(pipeline, volume_value, &url).await?;

    pipeline.set_state(gstreamer::State::Playing).map_err(|_| {
        ApplicationError::StateError("Failed to set pipeline to Playing".to_string())
    })?;
    Ok(())
}

/// 设置 pipeline 的 uri 和 headers
async fn set_pipeline_uri_with_headers(
    pipeline: &gstreamer::Pipeline,
    volume_value: f64,
    url: &str,
) -> Result<(), ApplicationError> {
    let source = gstreamer::ElementFactory::make("souphttpsrc")
        .build()
        .map_err(|_| {
            ApplicationError::ElementError("Failed to create souphttpsrc element".to_string())
        })?;
    source.set_property("location", url);

    let mut headers = gstreamer::Structure::new_empty("headers");
    headers.set(
        "User-Agent",
        "Mozilla/5.0 BiliDroid/..* (bbcallen@gmail.com)",
    );
    headers.set("Referer", "https://www.bilibili.com");
    source.set_property("extra-headers", &headers);

    let decodebin = gstreamer::ElementFactory::make("decodebin")
        .build()
        .map_err(|_| {
            ApplicationError::ElementError("Failed to create decodebin element".to_string())
        })?;
    // let initial_volume = 0.10f64; // 音量值
    // let set_volume = (0.10f64).clamp(0.0, 2.0); // 音量值
    let volume = gstreamer::ElementFactory::make("volume")
        .property("volume", volume_value)
        .name("audio_volume") // 设置名称方便查找
        .build()
        .map_err(|e| {
            tracing::error!("Failed to create volume: {:?}", e);
            ApplicationError::ElementError("Failed to set volume element".to_string())
        })?;
    // 检查volume元素是否支持音量属性
    // let props = volume.list_properties();
    // for prop in props {
    //     tracing::debug!("Volume element property: {}", prop.name());
    // }
    pipeline
        .add_many([&source, &decodebin, &volume])
        .map_err(|_| {
            ApplicationError::PipelineError("Failed to add elements to pipeline".to_string())
        })?;
    source.link(&decodebin).map_err(|_| {
        ApplicationError::LinkError("Failed to link source to decodebin".to_string())
    })?;

    let pipeline_weak = pipeline.downgrade();
    // 使用一个Weak引用跟踪volume元素
    let volume_weak = gstreamer::prelude::ObjectExt::downgrade(&volume);
    decodebin.connect_pad_added(move |_decodebin, src_pad| {
        // if let Some(pipeline) = pipeline_weak.upgrade() {
        //     // 创建音频处理链元素
        //     let audioconvert = gstreamer::ElementFactory::make("audioconvert")
        //         .build()
        //         .expect("Failed to create audioconvert element");
        //     // 音频采样
        //     let audioresample = gstreamer::ElementFactory::make("audioresample")
        //         .build()
        //         .expect("Failed to create audioresample element");
        //     //
        //     let autoaudiosink = gstreamer::ElementFactory::make("autoaudiosink")
        //         .build()
        //         .expect("Failed to create autoaudiosink element");
        //     // 创建并设置volume元素 - 在音频链中
        //     let set_volume = 0.10f64; // 10% 音量
        //     // 音量控制
        //     let volume = gstreamer::ElementFactory::make("volume")
        //         .property("volume", set_volume)
        //         .build()
        //         .expect("Failed to create volume element");

        //     pipeline
        //         .add_many([&audioconvert, &audioresample, &autoaudiosink, &volume])
        //         .expect("Failed to add elements to pipeline");

        //     audioconvert
        //         .sync_state_with_parent()
        //         .expect("Failed to sync_state_with_parent for audioconvert");
        //     audioresample
        //         .sync_state_with_parent()
        //         .expect("Failed to sync_state_with_parent for audioresample");
        //     autoaudiosink
        //         .sync_state_with_parent()
        //         .expect("Failed to sync_state_with_parent for autoaudiosink");

        //     let audio_pad = audioconvert
        //         .static_pad("sink")
        //         .expect("Failed to get static pad");
        //     src_pad.link(&audio_pad).expect("Failed to link pads");

        //     audioconvert
        //         .link(&audioresample)
        //         .expect("Failed to link audioconvert to audioresample");
        //     audioresample
        //         .link(&autoaudiosink)
        //         .expect("Failed to link audioresample to autoaudiosink");

        //     tracing::info!("Pipeline elements linked successfully");
        // } else {
        //     tracing::error!("Failed to upgrade pipeline reference");
        // }

        if let (Some(pipeline), Some(volume)) = (pipeline_weak.upgrade(), volume_weak.upgrade()) {
            // 检查pad是否为音频
            let caps = src_pad.current_caps();
            if let Some(caps) = caps {
                if let Some(structure) = caps.structure(0) {
                    let name = structure.name();
                    // tracing::info!("Pad caps name: {}", name);

                    if name.starts_with("audio/") {
                        // tracing::info!("Audio pad detected, building audio chain");
                        // 创建audio chain
                        let audioconvert =
                            match gstreamer::ElementFactory::make("audioconvert").build() {
                                Ok(el) => el,
                                Err(e) => {
                                    tracing::error!("Failed to create audioconvert: {:?}", e);
                                    return;
                                }
                            };

                        let audioresample =
                            match gstreamer::ElementFactory::make("audioresample").build() {
                                Ok(el) => el,
                                Err(e) => {
                                    tracing::error!("Failed to create audioresample: {:?}", e);
                                    return;
                                }
                            };

                        let autoaudiosink =
                            match gstreamer::ElementFactory::make("autoaudiosink").build() {
                                Ok(el) => el,
                                Err(e) => {
                                    tracing::error!("Failed to create autoaudiosink: {:?}", e);
                                    return;
                                }
                            };

                        // 添加到pipeline
                        if pipeline
                            .add_many([&audioconvert, &audioresample, &autoaudiosink])
                            .is_err()
                        {
                            tracing::error!("Failed to add audio elements to pipeline");
                            return;
                        }

                        // 确保volume已经在pipeline中
                        if volume.parent().is_none() && pipeline.add(&volume).is_err() {
                            tracing::error!("Failed to add volume to pipeline");
                        }

                        // 同步状态
                        let _ = audioconvert.sync_state_with_parent();
                        let _ = audioresample.sync_state_with_parent();
                        let _ = autoaudiosink.sync_state_with_parent();
                        let _ = volume.sync_state_with_parent();

                        // 获取audioconvert的sink pad
                        let audio_pad = match audioconvert.static_pad("sink") {
                            Some(pad) => pad,
                            None => {
                                tracing::error!("Failed to get audioconvert sink pad");
                                return;
                            }
                        };

                        // 链接所有元素
                        if src_pad.link(&audio_pad).is_err() {
                            tracing::error!("Failed to link decodebin to audioconvert");
                            return;
                        }

                        if audioconvert.link(&volume).is_err() {
                            tracing::error!("Failed to link audioconvert to volume");
                            return;
                        }

                        if volume.link(&audioresample).is_err() {
                            tracing::error!("Failed to link volume to audioresample");
                            return;
                        }

                        if audioresample.link(&autoaudiosink).is_err() {
                            tracing::error!("Failed to link audioresample to sink");
                            return;
                        }

                        // tracing::info!("Audio chain linked successfully");

                        // 双重检查音量设置
                        let vol_value: f64 = volume.property("volume");
                        tracing::info!("Volume element property value: {:.2}", vol_value);
                    } else {
                        tracing::info!("Non-audio pad, ignoring");
                    }
                }
            } else {
                tracing::error!("Failed to upgrade weak references");
            }
        }
    });

    // pipeline.set_state(gstreamer::State::Playing).map_err(|_| {
    //     ApplicationError::StateError("Failed to set pipeline to Playing".to_string())
    // })?;
    tracing::info!("Setting pipeline to Playing state");
    match pipeline.set_state(gstreamer::State::Playing) {
        Ok(_) => {
            tracing::info!("Pipeline set to Playing successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to set pipeline to Playing: {:?}", e);
            Err(ApplicationError::StateError(
                "Failed to set pipeline to Playing".to_string(),
            ))
        }
    }
}
