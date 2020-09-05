use byte_slice_cast::*;
use cpal::traits::StreamTrait;
use cpal::{
    traits::{DeviceTrait, HostTrait},
    SampleFormat,
};
use std::net::UdpSocket;

const MAX_SAMPLES_PER_PACKET: usize = 512;

fn main() {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("no output device available");

    let mut input_supported_formats_range = input_device
        .supported_input_configs()
        .expect("error while querying formats");
    let input_format = input_supported_formats_range
        .find(|f| f.sample_format() == SampleFormat::I16 && f.channels() == 1)
        .expect("no supported format?!")
        .with_sample_rate(cpal::SampleRate(48_000));

    let port = 13337;
    let socket = UdpSocket::bind(("0.0.0.0", 12345)).unwrap();
    socket.set_broadcast(true).unwrap();

    let input_config = input_format.into();
    let stream = input_device
        .build_input_stream(
            &input_config,
            move |data: &[i16], _info| {
                //println!("Got a buffer of size {}", data.len());
                for data in data.chunks(MAX_SAMPLES_PER_PACKET) {
                    let bytes = data.as_byte_slice();
                    //println!("Sending call, {} bytes", bytes.len());
                    let n = socket.send_to(&bytes, ("255.255.255.255", port)).unwrap();
                    if n != bytes.len() {
                        panic!("Sent the wrong number of bytes {}", n);
                    } else {
                        // Do nothing because we sent the number of bytes we expected to send
                    }
                }
            },
            |e| panic!("Error! {:?}", e),
        )
        .unwrap();

    stream.play().unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
