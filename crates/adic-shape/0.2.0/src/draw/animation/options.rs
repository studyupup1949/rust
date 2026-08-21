use std::time::Duration;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
/// Animation options, set separately from the controls
pub struct AnimationOptions {
    /// Should the animation start as playing or paused
    pub should_auto_start: bool,
    /// Should the animation loop upon completion
    pub should_loop: bool,
    /// Should the animation slider be shown
    pub show_slider: bool,
    /// Should the play/pause and reset buttons be shown
    pub show_play_reset: bool,
    /// Should the skip forward and backward buttons be shown
    pub show_skip: bool,
    /// Should the display for the frame be shown
    pub show_frame_label: bool,
    /// Time between animation ticks
    pub tick_time: Duration,
}


impl Default for AnimationOptions {
    fn default() -> Self {
        AnimationOptions {
            should_auto_start: false,
            should_loop: true,
            show_slider: true,
            show_play_reset: true,
            show_skip: true,
            show_frame_label: true,
            tick_time: Duration::from_millis(500),
        }
    }
}
