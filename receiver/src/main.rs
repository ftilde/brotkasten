use byte_slice_cast::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::net::UdpSocket;

include!("../../config.rs");

const SINGLE_SAMPLE_SIZE: usize = std::mem::size_of::<config::Sample>();

#[repr(C, align(2))] //Depends on Sample!!
struct PacketBuffer([u8; config::MAX_PACKET_SIZE]);

fn main() {
    let host = cpal::default_host();
    let output_device = host
        .default_output_device()
        .expect("no output device available");

    let mut output_supported_formats_range = output_device
        .supported_output_configs()
        .expect("error while querying formats");
    let output_format = output_supported_formats_range
        .find(|f| f.sample_format() == config::SAMPLE_FORMAT && f.channels() == 1)
        .expect("no supported format?!")
        .with_sample_rate(cpal::SampleRate(config::SAMPLE_RATE as _));

    let output_config = output_format.into();

    let socket = UdpSocket::bind((config::RECEIVER_BIND_ADDR, config::TRANSMISSION_PORT)).unwrap();
    socket.set_nonblocking(true).unwrap();

    let mut recv_buf = unsafe { std::mem::zeroed::<PacketBuffer>() };
    let mut recv_data_pos = 0;
    let mut num_received = 0;
    let stream = output_device
        .build_output_stream(
            &output_config,
            move |data: &mut [config::Sample], _info| {
                let mut data_pos = 0;
                loop {
                    let recv_data = &recv_buf.0[..].as_slice_of::<config::Sample>().unwrap();
                    while recv_data_pos < num_received {
                        if data_pos == data.len() {
                            return;
                        }

                        data[data_pos] = recv_data[recv_data_pos];

                        data_pos += 1;
                        recv_data_pos += 1;
                    }
                    recv_data_pos = 0;
                    num_received = 0;

                    let n = match socket.recv(&mut recv_buf.0) {
                        Ok(n) => n,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            for i in data_pos..data.len() {
                                data[i] = 0;
                            }
                            eprintln!("No Packet: Underrun!");
                            return;
                        }
                        Err(e) => panic!("Recv error: {}", e),
                    };
                    assert!(
                        n % SINGLE_SAMPLE_SIZE == 0,
                        "Packet contains incomplete sample"
                    );
                    num_received = n / SINGLE_SAMPLE_SIZE;
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
