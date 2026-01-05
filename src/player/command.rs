use crate::{
    pb::{AddPlaylistRequest, DeletedRequest, PlayBvidRequest, SetModelRequest, SetVolumeRequest},
    player::state::PlayerStateSnapshot,
};

#[derive(Debug)]
pub enum PlayerCommand {
    Play,
    PlayBvid(PlayBvidRequest),
    Pause,
    Next,
    Previous,
    Stop,
    SetModel(SetModelRequest),
    SetVolume(SetVolumeRequest),
    AddPlaylist(AddPlaylistRequest),
    Delete(DeletedRequest),
    GetState(tokio::sync::oneshot::Sender<PlayerStateSnapshot>),
    ShowPlaylist(),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PlayMode {
    #[default]
    Normal,
    Shuffle,
    Repeat,
}
impl PlayMode {
    pub fn get_string(&self) -> String {
        match self {
            PlayMode::Normal => "顺序播放".to_string(),
            PlayMode::Shuffle => "随机播放".to_string(),
            PlayMode::Repeat => "单曲循环".to_string(),
        }
    }
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "shuffle" => Some(PlayMode::Shuffle),
            "repeat" => Some(PlayMode::Repeat),
            _ => Some(PlayMode::Normal),
        }
    }
}
