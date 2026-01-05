use crate::{
    errors::ApplicationError,
    player::{command::PlayMode, state::Music},
};
use once_cell::sync::Lazy;
use rand::seq::IteratorRandom;
use tokio::sync::Mutex;
// 播放列表
pub static PLAYLIST: Lazy<Mutex<Result<Playlist, ApplicationError>>> =
    Lazy::new(|| Mutex::new(Ok(Playlist { musics: Vec::new() })));

// 当前播放的音乐索引
pub static CURRENT_MUSIC_INDEX: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

/// 播放列表
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Playlist {
    pub musics: Vec<Music>,
}

impl Playlist {
    pub async fn add_musics() -> Result<Self, ApplicationError> {
        let musics = vec![
            Music {
                bvid: "BV1oZqqBZEGZ".into(),
                title: "西楼别序".into(),
                cid: "34856567673".into(),
                owner: "夕照影音".into(),
            },
            Music {
                bvid: "BV1Xa411R7uJ".into(),
                title: "长大成人".into(),
                cid: "806047205".into(),
                owner: "忆江南音乐".into(),
            },
            Music {
                bvid: "BV1w34y1K7dv".into(),
                title: "一花一世界".into(),
                cid: "1287078024".into(),
                owner: "荃球音悦台".into(),
            },
            Music {
                bvid: "BV1r7411p7R4".into(),
                title: "青花瓷".into(),
                cid: "321818216".into(),
                owner: "zyl2012_音乐无限".into(),
            },
        ];
        // 加载音乐列表
        Ok(Playlist { musics })
    }
    /// 获取当前播放的音乐
    pub async fn get_current_music(&self, index: usize) -> Result<Music, ApplicationError> {
        self.musics
            .get(index)
            .cloned() // Clone the music to return an owned value
            .ok_or_else(|| {
                ApplicationError::DataParsingError("Music index out of bounds".to_string())
            })
    }
    /// 移动到下一首音乐
    pub async fn move_to_next_music(
        &mut self,
        play_mode: PlayMode,
    ) -> Result<usize, ApplicationError> {
        // 获取当前播放的音乐索引
        let mut current_index = CURRENT_MUSIC_INDEX.lock().await;
        // 根据播放模式来确定下一首
        match play_mode {
            // 顺序播放
            PlayMode::Normal => {
                *current_index = (*current_index + 1) % self.musics.len();
            }
            // 随机播放
            PlayMode::Shuffle => {
                let mut rng = rand::rng();
                *current_index = (0..self.musics.len()).choose(&mut rng).ok_or_else(|| {
                    ApplicationError::DataParsingError("Failed to choose random music".to_string())
                })?;
            }
            // 单曲循环
            PlayMode::Repeat => {
                // Do nothing, keep the current index
            }
        }
        Ok(*current_index)
    }

    /// 移动到上一首音乐
    pub async fn move_to_previous_music(
        &mut self,
        play_mode: PlayMode,
    ) -> Result<usize, ApplicationError> {
        // 获取当前播放的音乐索引
        let mut current_index = CURRENT_MUSIC_INDEX.lock().await;
        // 根据播放模式来确定下一首
        match play_mode {
            PlayMode::Normal => {
                if *current_index == 0 {
                    *current_index = self.musics.len() - 1;
                } else {
                    *current_index -= 1;
                }
            }
            PlayMode::Shuffle => {
                let mut rng = rand::rng();
                *current_index = (0..self.musics.len()).choose(&mut rng).ok_or_else(|| {
                    ApplicationError::DataParsingError("Failed to choose random music".to_string())
                })?;
            }
            PlayMode::Repeat => {
                // Do nothing, keep the current index
            }
        }
        Ok(*current_index)
    }
    /// 获取当前播放的音乐索引
    pub async fn find_music_index(&self, bvid: &str) -> Option<usize> {
        self.musics.iter().position(|music| music.bvid == bvid)
    }
}
/// 加载播放列表
pub async fn load_playlist() -> Result<(), ApplicationError> {
    // 添加音乐
    let playlist = Playlist::add_musics().await?;
    // 加载播放列表
    let mut playlist_lock = PLAYLIST.lock().await;
    *playlist_lock = Ok(playlist); // Replace the old playlist with the new one
    Ok(())
}
/// 获取当前播放的音乐
pub async fn get_current_music() -> Result<Music, ApplicationError> {
    let playlist = PLAYLIST.lock().await;
    let playlist = playlist.as_ref().map_err(|e| e.clone())?;
    let index = *CURRENT_MUSIC_INDEX.lock().await;
    playlist.get_current_music(index).await
}
/// 移动到下一首音乐
pub async fn move_to_next_music(play_mode: PlayMode) -> Result<usize, ApplicationError> {
    let mut playlist = PLAYLIST.lock().await;
    let playlist = playlist.as_mut().map_err(|e| e.clone())?;
    playlist.move_to_next_music(play_mode).await
}
/// 移动到上一首音乐
pub async fn move_to_previous_music(play_mode: PlayMode) -> Result<usize, ApplicationError> {
    let mut playlist = PLAYLIST.lock().await;
    let playlist = playlist.as_mut().map_err(|e| e.clone())?;
    playlist.move_to_previous_music(play_mode).await
}
/// 设置当前播放的音乐索引
pub async fn set_current_music_index(index: usize) -> Result<(), ApplicationError> {
    let mut current_index = CURRENT_MUSIC_INDEX.lock().await;
    *current_index = index;
    Ok(())
}
