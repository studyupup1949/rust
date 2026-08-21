#[derive(Debug, Clone)]
/// Animation frame with time, data, and optionally a frame label
pub struct Frame<D> {
    /// Time marker in animation
    pub time: u32,
    /// Frame data
    pub data: D,
    /// Frame label
    pub label: Option<String>,
}

impl<D, DI> From<(u32, DI)> for Frame<D>
where DI: Into<D> {
    fn from(value: (u32, DI)) -> Self {
        Self { time: value.0, data: value.1.into(), label: None }
    }
}

impl<D, DI, S> From<(u32, DI, S)> for Frame<D>
where DI: Into<D>, S: Into<String> {
    fn from(value: (u32, DI, S)) -> Self {
        Self { time: value.0, data: value.1.into(), label: Some(value.2.into()) }
    }
}

impl<D> PartialEq for Frame<D> {
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time)
    }
}
impl<D> Eq for Frame<D> { }

impl<D> Ord for Frame<D> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}
impl<D> PartialOrd for Frame<D> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}



#[derive(Debug, Clone)]
/// `FrameReel` is a simple animated object, a list of u32 times and [`Frames`](Frame)
pub struct FrameReel<D> {
    /// [`Frame`]s in the `FrameReel`, sorted
    frames: Vec<Frame<D>>,
    /// Maximum time to animate
    reel_time: u32,
}


impl<D> FrameReel<D> {

    /// Constructor
    pub fn new<F, V>(frames: V, reel_time: u32) -> Self
    where V: IntoIterator<Item=F>, F: Into<Frame<D>> {
        // Sort the frames
        let mut sorted_frames = frames.into_iter().map(Into::into).collect::<Vec<Frame<D>>>();
        sorted_frames.sort();
        Self {
            frames: sorted_frames,
            reel_time,
        }
    }

    /// Animate frame data linearly
    pub fn simple_linear<F, V>(frame_data: V) -> Self
    where V: IntoIterator<Item=F>, F: Into<D> {
        // Sort the frames
        let mut sorted_frames = frame_data.into_iter().enumerate().map(|(time, data)| {
            let time = u32::try_from(time).expect("usize -> u32 conversion");
            Frame::from((time, data))
        }).collect::<Vec<_>>();
        sorted_frames.sort();
        // Calculate reel time as number of frames
        let reel_time = sorted_frames.len().try_into().unwrap();
        Self {
            frames: sorted_frames,
            reel_time,
        }
    }

    /// Animate frame data linearly with labels
    pub fn simple_linear_labelled<F, V, S>(frame_data: V) -> Self
    where V: IntoIterator<Item=(F, S)>, F: Into<D>, S: Into<String> {
        // Sort the frames
        let mut sorted_frames = frame_data.into_iter().enumerate().map(|(time, (data, label))| {
            let time = u32::try_from(time).expect("usize -> u32 conversion");
            Frame::from((time, data, label))
        }).collect::<Vec<_>>();
        sorted_frames.sort();
        // Calculate reel time as number of frames
        let reel_time = sorted_frames.len().try_into().unwrap();
        Self {
            frames: sorted_frames,
            reel_time,
        }
    }

    /// An iterator of frame data from `beg_time` to `end_time`
    pub fn frame_vec(&self) -> &Vec<Frame<D>> {
        &self.frames
    }

    /// Number of frames in `FrameReel`
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Is the `FrameReel` empty
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Maximum time of the `FrameReel` before it completes or loops
    pub fn reel_time(&self) -> u32 {
        self.reel_time
    }

}
