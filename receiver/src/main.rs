use byte_slice_cast::*;
use cpal::traits::StreamTrait;
use cpal::{
    traits::{DeviceTrait, HostTrait},
    SampleFormat,
};
use std::net::UdpSocket;
use std::sync::atomic::Ordering;
use std::sync::Arc;

type Sample = i16;
const MAX_SAMPLES_PER_PACKET: usize = 512;
const SINGLE_SAMPLE_SIZE: usize = std::mem::size_of::<Sample>();
const MAX_PACKET_SIZE: usize = MAX_SAMPLES_PER_PACKET * SINGLE_SAMPLE_SIZE;
const SAMPLE_RATE: usize = 48000;

const RING_BUFFER_SIZE: usize = SAMPLE_RATE / 8;

#[repr(C, align(2))] //Depends on Sample!!
struct PacketBuffer([u8; MAX_PACKET_SIZE]);

fn main() {
    let host = cpal::default_host();
    let output_device = host
        .default_output_device()
        .expect("no output device available");

    let mut output_supported_formats_range = output_device
        .supported_output_configs()
        .expect("error while querying formats");
    let output_format = output_supported_formats_range
        .find(|f| f.sample_format() == SampleFormat::I16 && f.channels() == 1)
        .expect("no supported format?!")
        .with_sample_rate(cpal::SampleRate(SAMPLE_RATE as _));

    // Safety: Just plain old data
    let sample_buffer: [std::sync::atomic::AtomicI16; RING_BUFFER_SIZE] =
        unsafe { std::mem::zeroed() };

    let sample_source = Arc::new(sample_buffer);
    let mut source_pos = 0;
    let sample_sink = Arc::clone(&sample_source);
    let mut sink_pos = RING_BUFFER_SIZE / 2;

    let output_config = output_format.into();
    let stream = output_device
        .build_output_stream(
            &output_config,
            move |data: &mut [Sample], _info| {
                for o in data {
                    *o = sample_source[source_pos].load(Ordering::SeqCst);
                    source_pos = (source_pos + 1) % RING_BUFFER_SIZE;
                }
            },
            |e| panic!("Error! {:?}", e),
        )
        .unwrap();

    let port = 13337;
    let socket = UdpSocket::bind(("0.0.0.0", port)).unwrap();
    //socket.set_read_timeout(Some(Duration::new(5, 0))).unwrap();

    // Safety: Just u8s. Zeros are fine.
    let mut buf = unsafe { std::mem::zeroed::<PacketBuffer>() };

    stream.play().unwrap();
    loop {
        let n = socket.recv(&mut buf.0).unwrap();
        assert!(
            n % SINGLE_SAMPLE_SIZE == 0,
            "Packet contains incomplete sample"
        );

        let data = &buf.0[..n].as_slice_of::<Sample>().unwrap();
        //println!("Got {} samples", data.len());

        for s in data.iter() {
            sample_sink[sink_pos].store(*s, Ordering::SeqCst);
            sink_pos = (sink_pos + 1) % RING_BUFFER_SIZE;
        }
    }
}
