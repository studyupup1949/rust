#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Animation controls, e.g. `play_state` and `current_time`
pub struct AnimationControls {
    /// State of animation play/pause, set in controls
    pub play_state: PlayState,
    /// Current reel time
    pub current_time: u32,
}

impl Default for AnimationControls {
    fn default() -> Self {
        Self {
            play_state: PlayState::Paused,
            current_time: 0,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// State of animation play/pause
pub enum PlayState {
    /// Animation is playing
    Playing,
    /// Animation is paused
    Paused,
}

impl PlayState {

    /// Toggle between playing and paused
    pub fn toggle(&mut self) {
        match self {
            PlayState::Playing => {
                *self = Self::Paused;
            },
            PlayState::Paused => {
                *self = Self::Playing;
            },
        }
    }

}
