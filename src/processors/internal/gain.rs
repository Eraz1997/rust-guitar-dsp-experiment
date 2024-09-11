use crate::processors::Processor;

pub struct Gain {
    pub decimal: f32,
    range_db: (f32, f32),
}

impl Gain {
    pub fn set_db_range(&mut self, min: f32, max: f32) {
        self.range_db = (min, max);
    }

    #[inline(always)]
    pub fn get_linear_gain(&self) -> f32 {
        let (min_db, max_db) = self.range_db;
        let decibels = self.decimal * (max_db - min_db) + min_db;
        10.0_f32.powf(decibels / 20.0)
    }

    pub fn set_linear_gain(&mut self, gain: f32) {
        let (min_db, max_db) = self.range_db;
        let decibels = gain.log10() * 20.0;
        self.decimal = (decibels - min_db) / (max_db - min_db)
    }
}

impl Processor for Gain {
    fn new(_: &u32, _: &usize) -> Self
    where
        Self: Sized,
    {
        Self {
            decimal: 1.0,
            range_db: (1.0, 10.0),
        }
    }

    #[inline(always)]
    fn process(&mut self, data: &mut Vec<f32>) {
        let gain = self.get_linear_gain();
        for sample in data.iter_mut() {
            *sample *= gain
        }
    }
}
