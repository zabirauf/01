use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, FromSample};
use std::sync::{Arc, Mutex};
use std::io::BufWriter;
use std::fs::File;
use hound;

pub fn record_wav_audio(total_seconds: u64) ->  Result<&'static str, anyhow::Error>  {
    let host = cpal::default_host();

    println!("Discovering input devices");

    for d in host.input_devices().expect("discovery failure") {
        println!(
            "Device: {}",
            d.name().expect("Unable to find name of device")
        );
    }

    let device = host.default_input_device().expect("No input device found");
    let mut supported_configs_range = device
        .supported_input_configs()
        .expect("error while querying configs");
    let supported_config_range = supported_configs_range
        .next()
        .expect("no supported config?!");

    let min_sample_rate = supported_config_range.min_sample_rate();
    let max_sample_rate = supported_config_range.max_sample_rate();

    println!("Default device sample rate: {} - {}", min_sample_rate.0, max_sample_rate.0);
    println!("Default device sample format type: {}", supported_config_range.sample_format());

    let supported_config = supported_config_range
        .with_sample_rate(min_sample_rate);

    let config = supported_config.config();

    let spec = wav_spec_from_config(&supported_config);

    const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");

    //let mut inner_writer = std::io::Cursor::new(Vec::new());
    //let writer = hound::WavWriter::new(inner_writer, spec)?;
    let writer = hound::WavWriter::create(PATH, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    // A flag to indicate that recording is in progress.
    println!("Begin recording for {} secs...", total_seconds);

    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::I8 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i8, i8>(data, &writer_2),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i32, i32>(data, &writer_2),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
            err_fn,
            None,
        )?,
        sample_format => {
            return Err(anyhow::Error::msg(format!(
                "Unsupported sample format '{sample_format}'"
            )))
        }
    };

    stream.play()?;

    // Let recording go for roughly three seconds.
    std::thread::sleep(std::time::Duration::from_secs(total_seconds));
    drop(stream);

    // Finalize the WavWriter to write the WAV file headers
    writer.lock().unwrap().take().unwrap().finalize()?;

    println!("Recording complete! {}", PATH);
    // let inner_writer = writer.lock().unwrap().take().unwrap().into_inner()?;
    return Ok(PATH);
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}


//type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<std::io::Cursor<Vec<u8>>>>>>;
type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}