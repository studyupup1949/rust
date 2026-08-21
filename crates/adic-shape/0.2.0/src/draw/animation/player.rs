use std::fmt::Debug;
use super::FrameReel;


#[derive(Debug, Clone)]
/// Animation player, takes a [`FrameReel`] and plays it
pub struct AnimationPlayer<D>
where D: Debug + Clone {
    reel: FrameReel<D>,
    looping: bool,
    data: Option<D>,
    label: Option<String>,
    time: Option<u32>,
    frame_idx: Option<usize>,
    started: bool,
}


impl<D> AnimationPlayer<D>
where D: Debug + Clone {

    /// Create a new player with the given [`FrameReel`]
    pub fn new(reel: FrameReel<D>, should_loop: bool) -> Self {

        assert!(
            !reel.is_empty() && reel.reel_time() != 0,
            "FrameReel cannot be empty and reel_length cannot be zero"
        );

        Self {
            reel,
            looping: should_loop,
            data: None,
            label: None,
            time: None,
            frame_idx: None,
            started: false,
        }

    }

    /// Set whether the player should loop
    pub fn set_looping(&mut self, should_loop: bool) {
        self.looping = should_loop;
        self.reset();
    }

    /// Start the animation
    pub fn start(&mut self) {

        self.data = None;
        self.label = None;
        self.time = Some(self.min_time());
        self.frame_idx = Some(0);
        self.started = true;

        self.set_frame_from_time(self.min_time());

    }

    /// Start the animation from the end
    pub fn start_back(&mut self) {

        self.data = None;
        self.label = None;
        self.time = Some(self.max_time());
        self.frame_idx = Some(self.reel.len() - 1);
        self.started = true;

        self.set_frame_from_time(self.max_time());

    }

    /// Tick forward the animation time
    pub fn tick(&mut self) {

        if !self.is_started() { return; }

        let max_time = self.max_time();
        let Some(time) = &mut self.time else {
            panic!("Animation started but no time set");
        };

        if *time == max_time {
            self.complete_or_loop();
            return;
        }

        let new_time = *time + 1;
        if self.frame_idx.is_none() {
            self.set_frame_from_time(new_time);
        } else {
            *time = new_time;
        }

        if let Some(mut idx) = self.frame_idx {
            while idx < self.reel.len() - 1 {
                let peek_frame = &self.reel.frame_vec()[idx+1];
                if peek_frame.time <= new_time {
                    idx += 1;
                    self.data = Some(peek_frame.data.clone());
                    self.label = peek_frame.label.clone();
                    self.frame_idx = Some(idx);
                } else {
                    break;
                }
            }
        }

    }

    /// Tick backward the animation time
    pub fn tick_back(&mut self) {

        if !self.is_started() { return; }

        let min_time = self.min_time();
        let Some(time) = &mut self.time else {
            panic!("Animation started but no time set");
        };

        if *time == min_time {
            self.complete_or_loop_back();
            return;
        }

        let new_time = *time - 1;
        if self.frame_idx.is_none() {
            self.set_frame_from_time(new_time);
        } else {
            *time = new_time;
        }

        if let Some(mut idx) = self.frame_idx {
            while idx > 0 && self.reel.frame_vec()[idx].time > new_time {
                idx -= 1;
            }
            if self.reel.frame_vec()[idx].time <= new_time {
                self.data = Some(self.reel.frame_vec()[idx].data.clone());
                self.label = self.reel.frame_vec()[idx].label.clone();
                self.frame_idx = Some(idx);
            } else {
                self.data = None;
                self.label = None;
                self.frame_idx = None;
            }
        }

    }

    /// Current data of the animation
    pub fn current_data(&self) -> Option<&D> {
        self.data.as_ref()
    }

    /// Current time of the animation
    pub fn current_time(&self) -> Option<u32> {
        self.time
    }

    /// Current frame of the animation, if any
    pub fn current_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Minimum time of the animation
    pub fn min_time(&self) -> u32 {
        self.reel.frame_vec().first().map_or(0, |f| f.time)
    }

    /// Maximum time of the animation
    pub fn max_time(&self) -> u32 {
        self.reel.reel_time() - 1
    }

    /// Is animation started
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Is animation complete
    pub fn is_completed(&self) -> bool {
        !self.looping && self.current_time().is_some_and(|t| t >= self.max_time())
    }


    /// Set frame by finding last frame that has time less than t
    pub fn set_frame_from_time(&mut self, t: u32) {
        // Set new data IF the frame reel has a frame less than time t
        self.time = Some(t);
        let f_idx_past_time = self.reel.frame_vec().iter().position(|f| f.time > t).unwrap_or(self.reel.len());
        if f_idx_past_time > 0 {
            let current_frame = &self.reel.frame_vec()[f_idx_past_time - 1];
            self.data = Some(current_frame.data.clone());
            self.label = current_frame.label.clone();
            self.frame_idx = Some(f_idx_past_time - 1);
        }
    }

    fn reset(&mut self) {
        self.data = None;
        self.label = None;
        self.time = None;
        self.frame_idx = None;
        self.started = false;
    }

    /// Complete animation or loop
    fn complete_or_loop(&mut self) {
        if self.looping {
            self.start();
        }
    }

    /// Complete animation or loop backward
    fn complete_or_loop_back(&mut self) {
        if self.looping {
            self.start_back();
        }
    }

}
