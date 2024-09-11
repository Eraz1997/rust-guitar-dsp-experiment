pub mod frontline;
pub mod internal;

pub trait Processor {
    fn new(sample_rate: &u32, buffer_size: &usize) -> Self
    where
        Self: Sized;
    fn process(&mut self, data: &mut Vec<f32>);
}
