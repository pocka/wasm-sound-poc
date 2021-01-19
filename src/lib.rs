use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(PartialEq)]
enum GateState {
    Closing,
    Opening,
    // (Remains in frame)
    Holding(u32),
    // (Remains in frame)
    Releasing(u32),
    // (Remains in frame)
    Attacking(u32),
}

/// Indicates how loud is the sample level.
/// Here, Open Threshold is always bigger than or equal to Close Threshold.
enum SampleLevel {
    /// Below both Open Threshold and Close Threshold
    Silent,
    /// Above or Equal to Close Threshold, but below Open Threshold
    AboveCloseThreshold,
    /// Above Open Threshold
    AboveOpenThreshold,
}

#[wasm_bindgen]
pub struct Gate {
    state: GateState,

    open_threshold: f32,
    close_threshold: f32,
}

#[wasm_bindgen]
impl Gate {
    pub fn new(open_threshold: f32, close_threshold: f32) -> Gate {
        Gate {
            state: GateState::Closing,
            open_threshold,
            close_threshold,
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> bool {
        use GateState::*;
        use SampleLevel::*;

        let level = self.get_sample_level(input);

        log(match level {
            AboveOpenThreshold => "Open",
            AboveCloseThreshold => "Close",
            Silent => "Silent",
        });

        let next_state = match (&self.state, level) {
            (Closing, AboveOpenThreshold) => Some(Attacking(5)),
            (Attacking(remains), _) if *remains <= 0 => Some(Opening),
            (Opening, Silent) => Some(Holding(5)),
            (Holding(_), AboveCloseThreshold) => Some(Opening),
            (Holding(remains), _) if *remains <= 0 => Some(Releasing(5)),
            (Releasing(remains), AboveOpenThreshold) => Some(Attacking(5 * (1 - remains / 2))),
            (Releasing(remains), _) if *remains <= 0 => Some(Closing),
            _ => None,
        };

        if let Some(state) = next_state {
            self.state = state;
        }

        // Copy input buffer into output buffer, while applying gate.
        if self.state != Closing {
            for frame in 0..input.len() {
                output[frame] = input[frame];
            }
        }

        true
    }

    /// Given samples in a channel, returns the level of the loudest sample.
    fn get_sample_level(&self, samples: &[f32]) -> SampleLevel {
        // the loudest sample
        let max = samples
            .iter()
            .map(|s| s.abs())
            .fold(0.0, |a, b| if a > b { a } else { b });

        if max >= self.open_threshold {
            SampleLevel::AboveOpenThreshold
        } else if max >= self.close_threshold {
            SampleLevel::AboveCloseThreshold
        } else {
            SampleLevel::Silent
        }
    }
}
