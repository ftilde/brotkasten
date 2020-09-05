use byte_slice_cast::*;
use cpal::traits::StreamTrait;
use cpal::{
    traits::{DeviceTrait, HostTrait},
    SampleFormat,
};
use std::net::UdpSocket;

type Sample = i16;
const MAX_SAMPLES_PER_PACKET: usize = 512;
const SINGLE_SAMPLE_SIZE: usize = std::mem::size_of::<Sample>();
const MAX_PACKET_SIZE: usize = MAX_SAMPLES_PER_PACKET * SINGLE_SAMPLE_SIZE;
const SAMPLE_RATE: usize = 48000;

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

    let output_config = output_format.into();

    let port = 13337;
    let socket = UdpSocket::bind(("0.0.0.0", port)).unwrap();
    socket.set_nonblocking(true).unwrap();

    let mut recv_buf = unsafe { std::mem::zeroed::<PacketBuffer>() };
    let mut recv_data_pos = 0;
    let mut num_received = 0;
    let stream = output_device
        .build_output_stream(
            &output_config,
            move |data: &mut [Sample], _info| {
                let mut data_pos = 0;
                loop {
                    let recv_data = &recv_buf.0[..].as_slice_of::<Sample>().unwrap();
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
