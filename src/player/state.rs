// 用来存放音乐数据
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Default)]
pub struct Music {
    pub bvid: String,
    pub cid: String,
    pub title: String,
    pub owner: String,
}
impl std::fmt::Debug for Music {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} - {}", self.title, self.owner, self.bvid)
    }
}
// 播放器快照，用于返回状态信息
#[derive(Debug, Clone)]
pub struct PlayerStateSnapshot {
    pub current_music: Option<Music>,  // 当前播放的音乐
    pub is_playing: bool,              // 是否正在播放
    pub play_mode: String,             // 播放模式
    pub current_index: Option<usize>,  // 当前播放的索引
    pub playlist_len: usize,           // 播放列表长度
    pub current_position: Option<f64>, // 当前播放位置 (秒)
    pub duration: Option<f64>,         // 当前音乐总时长 (秒)
}

impl std::fmt::Display for PlayerStateSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "当前播放: {} {:?}/{:?} 播放模式: {} 第{}个/总共{}",
            self.current_music
                .as_ref()
                .map(|m| m.title.clone())
                .unwrap_or_default(),
            self.current_position,
            self.duration,
            self.play_mode,
            self.current_index.unwrap_or_default() + 1,
            self.playlist_len, // todo()!
        )
    }
}
