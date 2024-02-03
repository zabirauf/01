mod transcribe;
mod record;

use clap::Parser;
use std::path::PathBuf;
use transcribe::transcribe;
use record::record_wav_audio;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// This is the model for Whisper STT
    #[arg(short, long, value_parser, required = true)]
    model_path: PathBuf,
    
    /// This is the wav audio file that will be converted from speech to text
    #[arg(short, long, value_parser)]
    file_path: Option<PathBuf>,
}

fn main() {

    let args = Args::parse();

    println!("Model: {}", args.model_path.display());

    let file_path = match args.file_path {
        Some(fp) => fp,
        None => PathBuf::from(record_wav_audio(3).expect("Error recording"))
    };

    let result = transcribe(&args.model_path, &file_path);

    match result {
        Ok(transcription) => println!("Transcription:\n{}", transcription),
        Err(e) => println!("Error: {}", e),
    }
}
