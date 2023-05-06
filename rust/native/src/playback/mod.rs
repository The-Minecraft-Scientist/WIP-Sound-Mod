use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    ChannelCount, Device, SampleFormat, SampleRate, SupportedBufferSize, SupportedStreamConfig,
};
use std::ops::Div;
use std::thread;
use std::time::Duration;
use symphonia::core::conv::FromSample;

pub struct Player {
    device: Device,
}
impl Player {}
