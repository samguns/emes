use crate::error::{PlayerError, Result};
use ffmpeg_next::{
    format::{self, input},
    media::Type,
    software::resampling::context::Context as ResampleContext,
    util::frame::audio::Audio as AudioFrame,
    codec, decoder, format::sample::Sample, ChannelLayout,
};
use std::path::Path;
// use tokio::sync::mpsc;
use tracing::{debug, info};

pub struct AudioDecoder {
    format_context: format::context::Input,
    decoder: decoder::Audio,
    stream_index: usize,
    resampler: Option<ResampleContext>,
    sample_rate: u32,
    channels: u16,
    duration: f64,
}

#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub data: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub timestamp: f64,
}

impl AudioDecoder {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Initialize FFmpeg
        ffmpeg_next::init().map_err(|e| PlayerError::Ffmpeg(e))?;
        
        let path = path.as_ref();
        info!("Opening audio file: {:?}", path);
        
        let format_context = input(&path)
            .map_err(|e| PlayerError::Ffmpeg(e))?;
        
        // Find the first audio stream
        let stream = format_context
            .streams()
            .best(Type::Audio)
            .ok_or(PlayerError::NoAudioTracks)?;
        
        let stream_index = stream.index();
        
        // Get codec parameters
        let codec_parameters = stream.parameters();
        let _codec = ffmpeg_next::decoder::find(codec_parameters.id())
            .ok_or_else(|| PlayerError::UnsupportedCodec(format!("{:?}", codec_parameters.id())))?;
        
        // Create decoder context
        let context = codec::context::Context::from_parameters(codec_parameters.clone())
            .map_err(|e| PlayerError::Ffmpeg(e))?;
        let mut decoder = context.decoder().audio()
            .map_err(|e| PlayerError::Ffmpeg(e))?;
        
        // Set decoder parameters
        decoder.set_parameters(codec_parameters)
            .map_err(|e| PlayerError::Ffmpeg(e))?;
        
        let sample_rate = decoder.rate();
        let channels = decoder.channels() as u16;
        
        // Calculate duration
        let duration = if stream.duration() > 0 {
            stream.duration() as f64 * f64::from(stream.time_base())
        } else {
            format_context.duration() as f64 / 1_000_000.0
        };
        
        info!("Audio file opened: {}Hz, {} channels, {:.2}s duration", 
              sample_rate, channels, duration);
        
        Ok(AudioDecoder {
            format_context,
            decoder,
            stream_index,
            resampler: None,
            sample_rate,
            channels,
            duration,
        })
    }
    
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    pub fn channels(&self) -> u16 {
        self.channels
    }
    
    pub fn duration(&self) -> f64 {
        self.duration
    }
    
    pub fn seek(&mut self, timestamp: f64) -> Result<()> {
        let stream = self.format_context.stream(self.stream_index).unwrap();
        let time_base = stream.time_base();
        let ts = (timestamp / f64::from(time_base)) as i64;
        
        self.format_context.seek(ts, ..ts)
            .map_err(|e| PlayerError::Ffmpeg(e))?;
        
        self.decoder.flush();
        
        debug!("Seeked to timestamp: {:.2}s", timestamp);
        Ok(())
    }
    
    pub fn decode_stream_sync(&mut self, tx: std::sync::mpsc::Sender<AudioChunk>) -> Result<()> {
        let mut frame = AudioFrame::empty();
        
        // Setup resampler for consistent output format
        let target_sample_rate = 44100;
        let target_channels = 2;
        let target_format = Sample::F32(format::sample::Type::Planar);
        
        if self.sample_rate != target_sample_rate || 
           self.channels != target_channels ||
           self.decoder.format() != target_format {
            
            let resampler = ResampleContext::get(
                self.decoder.format(),
                self.decoder.channel_layout(),
                self.sample_rate,
                target_format,
                ChannelLayout::STEREO,
                target_sample_rate,
            ).map_err(|e| PlayerError::Ffmpeg(e))?;
            
            self.resampler = Some(resampler);
            info!("Resampler initialized: {}Hz {}ch -> {}Hz {}ch", 
                  self.sample_rate, self.channels, target_sample_rate, target_channels);
        }
        
        loop {
            match self.format_context.packets().next() {
                Some((stream, packet)) => {
                    if stream.index() != self.stream_index {
                        continue;
                    }
                    
                    self.decoder.send_packet(&packet)
                        .map_err(|e| PlayerError::Ffmpeg(e))?;
                    
                    while self.decoder.receive_frame(&mut frame).is_ok() {
                        let audio_chunk = self.process_frame(&frame)?;
                        
                        if tx.send(audio_chunk).is_err() {
                            debug!("Receiver dropped, stopping decode");
                            return Ok(());
                        }
                    }
                }
                None => break,
            }
        }
        
        // Flush remaining frames
        self.decoder.send_eof()
            .map_err(|e| PlayerError::Ffmpeg(e))?;
        
        while self.decoder.receive_frame(&mut frame).is_ok() {
            let audio_chunk = self.process_frame(&frame)?;
            
            if tx.send(audio_chunk).is_err() {
                break;
            }
        }
        
        debug!("Decoding completed");
        Ok(())
    }
    
    fn process_frame(&mut self, frame: &AudioFrame) -> Result<AudioChunk> {
        let timestamp = frame.timestamp().unwrap_or(0) as f64 * 
                       f64::from(self.format_context.stream(self.stream_index).unwrap().time_base());
        
        let (sample_rate, channels, data) = if let Some(ref mut resampler) = self.resampler {
            // Resample the frame
            let mut resampled_frame = AudioFrame::empty();
            resampler.run(&frame, &mut resampled_frame)
                .map_err(|e| PlayerError::Ffmpeg(e))?;
            
            let sample_rate = resampled_frame.rate();
            let channels = resampled_frame.channels() as u16;
            let data = self.extract_f32_samples(&resampled_frame)?;
            
            (sample_rate, channels, data)
        } else {
            // Use original frame
            let sample_rate = frame.rate();
            let channels = frame.channels() as u16;
            let data = self.extract_f32_samples(frame)?;
            
            (sample_rate, channels, data)
        };
        
        Ok(AudioChunk {
            data,
            sample_rate,
            channels,
            timestamp,
        })
    }
    
    fn extract_f32_samples(&self, frame: &AudioFrame) -> Result<Vec<f32>> {
        let format = frame.format();
        let channels = frame.channels() as usize;
        let samples_per_channel = frame.samples();
        
        match format {
            Sample::F32(format::sample::Type::Planar) => {
                let mut output = Vec::with_capacity(samples_per_channel * channels);
                
                for i in 0..samples_per_channel {
                    for ch in 0..channels {
                        let plane = frame.plane::<f32>(ch);
                        output.push(plane[i]);
                    }
                }
                
                Ok(output)
            },
            Sample::F32(format::sample::Type::Packed) => {
                let plane = frame.plane::<f32>(0);
                Ok(plane[..samples_per_channel * channels].to_vec())
            },
            Sample::I16(format::sample::Type::Planar) => {
                let mut output = Vec::with_capacity(samples_per_channel * channels);
                
                for i in 0..samples_per_channel {
                    for ch in 0..channels {
                        let plane = frame.plane::<i16>(ch);
                        output.push(plane[i] as f32 / 32768.0);
                    }
                }
                
                Ok(output)
            },
            Sample::I16(format::sample::Type::Packed) => {
                let plane = frame.plane::<i16>(0);
                let mut output = Vec::with_capacity(samples_per_channel * channels);
                
                for &sample in &plane[..samples_per_channel * channels] {
                    output.push(sample as f32 / 32768.0);
                }
                
                Ok(output)
            },
            _ => {
                Err(PlayerError::UnsupportedCodec(format!("Unsupported sample format: {:?}", format)))
            }
        }
    }
}