// I DON'T KNOW WHAT I'M DOING
// I've just copied the example from ac_ffmpeg and tried to make it work
// It seems to work at least at my machine so I'm happy

use ac_ffmpeg::{
    codec::{
        video::{self, PixelFormat, VideoEncoder, VideoFrameMut, VideoFrameScaler},
        Encoder,
    },
    format::{
        io::IO,
        muxer::{Muxer, OutputFormat},
    },
    time::{TimeBase, Timestamp},
    Error,
};
use std::{fs::File, time::Duration};

pub struct SimpleEncoder {
    encoder: VideoEncoder,
    muxer: Muxer<File>,
    frame_idx: i64,
    frame_timestamp: Timestamp,
    max_timestamp: Timestamp,
    time_base: TimeBase,
    source_pixel_format: PixelFormat,
    scaler: VideoFrameScaler,
}

impl SimpleEncoder {
    pub fn new(path: &str, width: u32, height: u32, secs: u64) -> Result<Self, Error> {
        let time_base = TimeBase::new(1, 60);

        let target_pixel_format = video::frame::get_pixel_format("yuv420p");
        let encoder = VideoEncoder::builder("libx264")?
            .pixel_format(target_pixel_format)
            .width(width as _)
            .height(height as _)
            .time_base(time_base)
            .build()?;

        let codec_parameters = encoder.codec_parameters().into();

        let output_format = OutputFormat::guess_from_file_name(path).ok_or_else(|| {
            Error::new(format!("unable to guess output format for file: {}", path))
        })?;

        let output = File::create(path)
            .map_err(|err| Error::new(format!("unable to create output file {}: {}", path, err)))?;

        let io = IO::from_seekable_write_stream(output);

        let mut muxer_builder = Muxer::builder();

        muxer_builder.add_stream(&codec_parameters)?;

        let source_pixel_format = video::frame::get_pixel_format("rgba");
        let scaler = VideoFrameScaler::builder()
            .source_pixel_format(source_pixel_format)
            .source_width(width as usize)
            .source_height(height as usize)
            .target_width(width as usize)
            .target_height(height as usize)
            .target_pixel_format(target_pixel_format)
            .build()?;

        let muxer = muxer_builder.build(io, output_format)?;
        let frame_idx = 0;
        let frame_timestamp = Timestamp::new(frame_idx, time_base);
        let max_timestamp = Timestamp::from_millis(0) + Duration::from_secs(secs);

        Ok(Self {
            encoder,
            muxer,
            frame_idx,
            frame_timestamp,
            max_timestamp,
            time_base,
            source_pixel_format,
            scaler,
        })
    }

    pub fn render(&mut self, data: &[u8]) -> Result<bool, Error> {
        let mut frame = VideoFrameMut::black(self.source_pixel_format, 1080 as _, 1080 as _)
            .with_time_base(self.time_base)
            .with_pts(self.frame_timestamp);
        let mut planes = frame.planes_mut();
        planes[0].data_mut().clone_from_slice(data);
        let frame = self.scaler.scale(&frame.freeze())?;

        self.encoder.push(frame)?;

        while let Some(packet) = self.encoder.take()? {
            self.muxer.push(packet.with_stream_index(0))?;
        }

        self.frame_idx += 1;
        self.frame_timestamp = Timestamp::new(self.frame_idx, self.time_base);

        Ok(self.frame_timestamp < self.max_timestamp)
    }

    pub fn done(mut self) -> Result<(), Error> {
        self.encoder.flush()?;

        while let Some(packet) = self.encoder.take()? {
            self.muxer.push(packet.with_stream_index(0))?;
        }

        self.muxer.flush()?;

        Ok(())
    }
}
