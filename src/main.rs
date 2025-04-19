use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::Producer, traits::Consumer, traits::Split, HeapRb};

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("no input device");
    let output_device = host.default_output_device().expect("no output device");

    let input_cfg = input_device
    .default_input_config()
    .unwrap_or_else(|e| {
        eprintln!("Input config error: {:?}", e);
        std::process::exit(1);
    });
    let output_cfg = output_device.default_output_config().expect("no input config");

    println!("Default input config: {:?}", input_cfg);
    println!("Default output config: {:?}", output_cfg);

    let sample_rate = input_cfg.sample_rate();
    let sample_format = input_cfg.sample_format();
    let channels = input_cfg.channels();

    let output_config = cpal::StreamConfig {
        channels,
        sample_rate,
        buffer_size: cpal::BufferSize::Default,
    };

    println!("sample rate: {}, format: {:?}, channels: {}", sample_rate.0, sample_format, channels);

    let buffer = HeapRb::<f32>::new(sample_rate.0 as usize / 20);
    let (mut producer, mut consumer) = buffer.split();

    let error_callback = |err| eprintln!("streaming error: {}", err);

    let input_stream = input_device.build_input_stream(
        &input_cfg.config(),
        move |data: &[f32], _| {
            for &sample in data {
                let _ = producer.try_push(sample);
            }
        },
        error_callback,
        None,
    )?;

    let output_stream = output_device.build_output_stream(
        &output_config,
        move |data: &mut [f32], _| {
            for sample in data.iter_mut() {
                *sample = consumer.try_pop().unwrap_or(0.0);
            }
        },
        error_callback,
        None,
    )?;

    input_stream.play()?;
    output_stream.play()?;

    print!("Start audio streaming...");
    std::thread::park();

    Ok(())
}
