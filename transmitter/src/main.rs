use byte_slice_cast::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::net::UdpSocket;

include!("../../config.rs");

fn main() {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("no output device available");

    let mut input_supported_formats_range = input_device
        .supported_input_configs()
        .expect("error while querying formats");
    let input_format = input_supported_formats_range
        .find(|f| f.sample_format() == config::SAMPLE_FORMAT && f.channels() == 1)
        .expect("no supported format?!")
        .with_sample_rate(cpal::SampleRate(config::SAMPLE_RATE as _));

    let socket =
        UdpSocket::bind((config::TRANSMITTER_BIND_ADDR, config::TRANSMITTER_BIND_PORT)).unwrap();
    socket.set_broadcast(true).unwrap();

    let input_config = input_format.into();
    let stream = input_device
        .build_input_stream(
            &input_config,
            move |data: &[config::Sample], _info| {
                //println!("Got a buffer of size {}", data.len());
                for data in
                    data.chunks(config::MAX_PACKET_SIZE / std::mem::size_of::<config::Sample>())
                {
                    let bytes = data.as_byte_slice();
                    //println!("Sending call, {} bytes", bytes.len());
                    let n = socket
                        .send_to(&bytes, ("255.255.255.255", config::TRANSMISSION_PORT))
                        .unwrap();
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
