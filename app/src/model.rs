//! UI model

use crate::consts::{MAX30102_NUM_SAMPLES, MAX30102_SAMPLE_RATE};
use cardiac_monitor_shared::{
    linreg::Linreg,
    signal::{Heartbeat, HeartbeatItr},
};
use heapless::Vec;

use heapless::binary_heap::{BinaryHeap, Max};

/// Single sample read from a sensor
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Max3012Sample {
    pub r: f32,
    pub ir: f32,
}

impl Max3012Sample {
    pub fn zero() -> Self {
        Max3012Sample { r: 0.0, ir: 0.0 }
    }
}

/// A chunk of sample data, represents one of the
/// channels (red or infrared)
pub struct Max3012SampleData {
    /// "AC" component of R/IR signal sample
    /// (sensor value - DC mean subtracted)
    pub ac: [f32; MAX30102_NUM_SAMPLES],

    /// "DC" mean of the sample
    dc_mean: f32,

    /// for scale, to display raw data
    pub ac_max: f32,
    pub ac_min: f32,

    linreg: Linreg<MAX30102_NUM_SAMPLES>,

    pub heartbeats: Vec<Heartbeat, 16>,

    pub heart_rate_bpm: Option<f32>,

    // Part of SPO2 formula, AC/DC
    ac_over_dc: f32,
}

impl Max3012SampleData {
    pub fn new() -> Self {
        Max3012SampleData {
            ac: [0.0; MAX30102_NUM_SAMPLES],
            dc_mean: 0.0,

            ac_max: 1.0,
            ac_min: 0.0,

            linreg: Linreg::new(),

            heartbeats: Vec::new(),

            heart_rate_bpm: None,

            ac_over_dc: 1.0,
        }
    }

    pub fn update_from_samples(&mut self, data: &[f32; MAX30102_NUM_SAMPLES]) {
        self.dc_mean = 0.0;
        self.ac_max = f32::MIN;
        self.ac_min = f32::MAX;

        for (i, x) in data.iter().enumerate() {
            self.ac[i] = *x;
            self.dc_mean += x;
        }
        self.dc_mean /= MAX30102_NUM_SAMPLES as f32;

        for ac in self.ac.iter_mut() {
            *ac -= self.dc_mean;
        }

        self.linreg.update_from(&self.ac);

        for (i, ac) in self.ac.iter_mut().enumerate() {
            *ac -= self.linreg.y(i as f32);
            self.ac_max = self.ac_max.max(*ac);
            self.ac_min = self.ac_min.min(*ac);
        }

        self.heartbeats.clear();

        self.ac_over_dc = 0.0;
        let mut hb_cnt = 0;

        // Keep track of distances (in array indexes) between heartbeats
        let mut hb_dist: BinaryHeap<usize, Max, 16> = BinaryHeap::new();
        let mut last_hb_idx: Option<usize> = None;
        let hb_threshold = (self.ac_max - self.ac_min) / 4.0;
        for hb in HeartbeatItr::new(&self.ac) {
            // Ignore small amplitude "wiggles", focus on larger transitions.
            // This only works if overall signal is clean enough from motion
            // artifacts (i.e. actual heartbeats stay relatively close to
            // min/max amplitude values).
            let hb_val_diff = hb.high_value - hb.low_value;

            if hb_val_diff > hb_threshold {
                let _ = self.heartbeats.push(hb);
                for lhb in last_hb_idx {
                    let _ = hb_dist.push(hb.high_idx - lhb);
                }

                last_hb_idx = Some(hb.high_idx);

                self.ac_over_dc += hb_val_diff;
                hb_cnt += 1;
            }
        }

        self.ac_over_dc = self.ac_over_dc / hb_cnt as f32 / self.dc_mean;

        self.heart_rate_bpm = None;

        if !hb_dist.is_empty() {
            // ignore extremes, pick a value in the middle
            for _ in 0..(hb_dist.len() / 2) {
                hb_dist.pop();
            }

            for hbd in hb_dist.pop() {
                self.heart_rate_bpm = Some(60.0 * MAX30102_SAMPLE_RATE.0 as f32 / hbd as f32);
            }
        }
    }
}

pub struct UIModel {
    pub r: Max3012SampleData,
    pub ir: Max3012SampleData,
}

impl UIModel {
    pub fn new() -> Self {
        UIModel {
            r: Max3012SampleData::new(),
            ir: Max3012SampleData::new(),
        }
    }

    pub fn update_from_samples(
        &mut self,
        oxi_r_samples: &[f32; MAX30102_NUM_SAMPLES],
        oxi_ir_samples: &[f32; MAX30102_NUM_SAMPLES],
    ) {
        self.r.update_from_samples(oxi_r_samples);
        self.ir.update_from_samples(oxi_ir_samples);
    }

    pub fn spo2(&self) -> f32 {
        let r_acdc = self.r.ac_over_dc;
        let ir_acdc = self.ir.ac_over_dc;
        let z = r_acdc / ir_acdc;
        (-45.06 * z + 30.354) * z + 94.845
    }
}
