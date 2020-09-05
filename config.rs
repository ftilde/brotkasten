#[allow(unused)]
mod config {
    pub type Sample = i16;
    pub const SAMPLE_FORMAT: cpal::SampleFormat = cpal::SampleFormat::I16;
    pub const MAX_PACKET_SIZE: usize = 1024; // Should be divisible by the size of Sample
    pub const SAMPLE_RATE: usize = 48000;
    pub const TRANSMITTER_BIND_ADDR: &str = "0.0.0.0";
    pub const RECEIVER_BIND_ADDR: &str = "0.0.0.0";
    pub const TRANSMISSION_PORT: u16 = 12345;
    pub const TRANSMITTER_BIND_PORT: u16 = 13337;
}
