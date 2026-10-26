use std::num::NonZero;
use std::time::Duration;

use rodio::buffer::SamplesBuffer;

const RATE: u32 = 44_100;
const NOTES: [(f32, f32); 2] = [(880., 0.), (1320., 0.14)];
const LENGTH: f32 = 0.3;
const VOLUME: f32 = 0.2;
const QUIET: f32 = 0.001;

fn samples() -> Vec<f32> {
    let rate = crate::num::real(RATE);
    let total = crate::num::index((0.14 + LENGTH) * rate);
    let mut out = vec![0.; total];
    for (frequency, start) in NOTES {
        let first = crate::num::index(start * rate);
        let count = crate::num::index(LENGTH * rate);
        for index in 0..count {
            let time = crate::num::real_usize(index) / rate;
            let gain = VOLUME * (QUIET / VOLUME).powf(time / LENGTH);
            if let Some(sample) = out.get_mut(first + index) {
                *sample += gain * (std::f32::consts::TAU * frequency * time).sin();
            }
        }
    }
    out
}

pub fn chime() {
    std::thread::spawn(|| {
        let (Some(channels), Some(rate)) = (NonZero::new(1), NonZero::new(RATE)) else {
            return;
        };
        match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(sink) => {
                sink.mixer()
                    .add(SamplesBuffer::new(channels, rate, samples()));
                std::thread::sleep(Duration::from_millis(600));
            }
            Err(error) => tracing::warn!(%error, "could not play the chime"),
        }
    });
}
