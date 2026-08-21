// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

use crate::shared;
pub use crate::shared::alsa::HZ_48K;
use crate::shared::alsa::*;

/// A Speaker connection.
pub struct Speaker {
    speaker: (usize, shared::alsa::pcm::PCM),
    speaker_buffer: Vec<i16>,
}

impl Speaker {
    /// Create a new Speaker connection. `id`s start at 0 and go up.
    ///
    /// # Usage
    /// ```
    /// let speaker = adi::speaker::Speaker::new(0, true /*stereo*/);
    /// ```
    pub fn new(id: u16, stereo: bool) -> Option<Self> {
        if id != 0 {
            return None;
        }

        lazy_init_alsa();

        let (speaker, speaker_buffer) = {
            let pcm = shared::alsa::pcm::PCM::new(
                context(),
                "default",
                shared::alsa::Direction::Playback,
            )
            .unwrap();
            set_settings(&pcm, stereo);
            let speaker_max_latency;
            (
                (
                    {
                        let hwp = pcm.hw_params_current(context()).unwrap();
                        let bs = hwp.get_buffer_size(context()).unwrap();

                        println!("Buffer Size: {}", bs);
                        speaker_max_latency = hwp.get_period_size(context()).unwrap() as usize * 2;

                        println!("PC: {}", hwp.get_channels(context()).unwrap());
                        println!("PR: {}", hwp.get_rate(context()).unwrap());

                        hwp.drop(context());
                        bs
                    },
                    pcm,
                ),
                vec![0i16; speaker_max_latency],
            )
        };

        speaker.1.prepare(context());

        Some(Self {
            speaker,
            speaker_buffer,
        })
    }

    /// Generate & push data to speaker output.  When a new sample is
    /// needed, closure `generator` will be called.  This should be called
    /// in a loop.
    pub fn update(&mut self, generator: &mut FnMut() -> i16) {
        let left = self.left() as usize;
        let write = if left < self.speaker_buffer.len() {
            self.speaker_buffer.len() - left
        } else {
            0
        };

        for i in 0..write {
            self.speaker_buffer[i] = generator();
        }

        self.push(&self.speaker_buffer[..write]);
    }

    /// Push data to the speaker output.
    fn push(&self, buffer: &[i16]) {
        if self
            .speaker
            .1
            .writei(context(), buffer)
            .unwrap_or_else(|_| 0)
            != buffer.len()
        {
            println!("buffer underrun!");

            self.speaker
                .1
                .recover(context(), 32, true)
                .unwrap_or_else(|x| panic!("ERROR: {}", x));

            if self
                .speaker
                .1
                .writei(context(), buffer)
                .unwrap_or_else(|_| 0)
                != buffer.len()
            {
                panic!("double buffer underrun!");
            }
        }
    }

    /// Get the number of samples left in the buffer.
    fn left(&self) -> usize {
        self.speaker.0
            - self
                .speaker
                .1
                .status(context())
                .unwrap()
                .get_avail(context())
    }
}

impl Drop for Speaker {
    fn drop(&mut self) {
        self.speaker.1.drop(context());
    }
}
