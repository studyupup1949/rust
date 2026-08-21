// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

use shared;
pub use shared::alsa::HZ_48K;
use shared::alsa::*;

/// A Microphone connection.
pub struct Mic {
    pcm: shared::alsa::pcm::PCM,
}

impl Mic {
    /// Create a new Microphone connection. `id`s start at 0 and go up.
    ///
    /// # Usage
    /// ```
    /// let mic = adi::mic::Mic::new(0);
    /// ```
    pub fn new(id: u16) -> Option<Self> {
        if id != 0 {
            return None;
        }

        lazy_init_alsa();

        let pcm =
            shared::alsa::pcm::PCM::new(context(), "plughw:0,0", shared::alsa::Direction::Capture)
                .unwrap();
        set_settings(&pcm, false /* microphones are never stereo */);
        {
            let hwp = pcm.hw_params_current(context()).unwrap();
            println!("CC: {}", hwp.get_channels(context()).unwrap());
            println!("CR: {}", hwp.get_rate(context()).unwrap());
            hwp.drop(context());
        }

        pcm.start(context());

        Some(Self { pcm })
    }

    /// Pull data from the microphone input.
    pub fn update(&self, buffer: &mut [i16]) -> usize {
        self.pcm.readi(context(), buffer).unwrap_or(0)
    }
}

impl Drop for Mic {
    fn drop(&mut self) {
        self.pcm.drop(context());
    }
}
