use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::{Producer, Consumer, Split, Observer}, HeapRb};

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("no input device");
    let output_device = host.default_output_device().expect("no output device");

    let input_cfg = input_device.default_input_config()?;
    let sample_rate = input_cfg.sample_rate();
    let channels = input_cfg.channels();
    let output_cfg = cpal::StreamConfig {
        channels,
        sample_rate,
        buffer_size: cpal::BufferSize::Default,
    };

    let buffer_frames = (sample_rate.0 as f32 * 0.03) as usize;
    let buffer = HeapRb::<f32>::new(buffer_frames);
    let (mut producer, mut consumer) = buffer.split();

    let error_callback = |err| eprintln!("streaming error: {}", err);

    let input_stream = input_device.build_input_stream(
        &input_cfg.config(),
        move |data: &[f32], _| {
            for &sample in data {
                if producer.is_full() {
                    eprintln!("Buffer overflow");
                }
                let _ = producer.try_push(sample);
            }
        },
        error_callback,
        None,
    )?;

    let output_stream = output_device.build_output_stream(
        &output_cfg,
        move |data: &mut [f32], _| {
            for sample in data.iter_mut() {
                if consumer.is_empty() {
                    eprintln!("Buffer underflow");
                }
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
