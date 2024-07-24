use std::{collections::HashMap, str::FromStr, sync::Arc};

// tips: I mostly sourced this from https://github.com/GStreamer/gst-examples/blob/master/webrtc/multiparty-sendrecv/gst-rust/src/main.rs

use anyhow::Ok;
use gio::{glib::{self}, subclass::prelude::ObjectImpl};
use gstreamer::{caps::NoFeature, prelude::*, subclass::prelude::{ElementImpl, GstObjectImpl}, GhostPad};
use gstreamer_rtp::{prelude::RTPHeaderExtensionExt, subclass::prelude::RTPHeaderExtensionImpl, RTPHeaderExtensionFlags};
use gstreamer_webrtc::{WebRTCDataChannel, WebRTCSessionDescription};
use once_cell::sync::Lazy;
use stellar_protocol::protocol::{EncodingPreset, PipelineOptimization};

use gio::subclass::prelude::ObjectSubclass;

use lazy_static::lazy_static;

use crate::streamerd::{build_capsfilter, StreamerConfig};

// https://gitlab.freedesktop.org/gstreamer/gstreamer/-/blob/main/subprojects/gst-examples/webrtc/sendrecv/gst-rust/src/main.rs#L30
const TWCC_URI: &str = "http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01";
const PLAYOUT_DELAY_URI: &str = "http://www.webrtc.org/experiments/rtp-hdrext/playout-delay";

#[derive(Default)]
pub struct PlayoutDelayRTPExperiment {}

pub mod inner {
    use gstreamer::glib;

    glib::wrapper! {
        pub struct PlayoutDelayRTPExperiment(ObjectSubclass<super::PlayoutDelayRTPExperiment>) @extends gstreamer_rtp::RTPHeaderExtension, gstreamer::Element, gstreamer::Object;
    }

    unsafe impl Send for PlayoutDelayRTPExperiment {}
    unsafe impl Sync for PlayoutDelayRTPExperiment {}
}

#[glib::object_subclass]
impl ObjectSubclass for PlayoutDelayRTPExperiment {
    const NAME: &'static str = "PlayoutDelayRTPExperiment";
    type Type = inner::PlayoutDelayRTPExperiment;
    type ParentType = gstreamer_rtp::RTPHeaderExtension;
}

impl ObjectImpl for PlayoutDelayRTPExperiment {}

impl GstObjectImpl for PlayoutDelayRTPExperiment {}

impl ElementImpl for PlayoutDelayRTPExperiment {
    fn metadata() -> Option<&'static gstreamer::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gstreamer::subclass::ElementMetadata> = Lazy::new(|| {
            gstreamer::subclass::ElementMetadata::new(
                "Custom RTP Header Extension for setting playout-delay",
                &gstreamer_rtp::RTP_HDREXT_ELEMENT_CLASS,
                "Custom RTP Header Extension for setting playout-delay",
                "todo <example@example.com>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }
}

// https://github.com/centricular/aes67-relay-chunker/blob/5e688f5c258b7a236cb25adc221184e915f05587/src/bin/rtp_hdr_ext/rtp_hdr_ext_ptp.rs#L55
impl RTPHeaderExtensionImpl for PlayoutDelayRTPExperiment {
    const URI: &'static str = "http://www.webrtc.org/experiments/rtp-hdrext/playout-delay";

    fn supported_flags(&self) -> RTPHeaderExtensionFlags {
        RTPHeaderExtensionFlags::ONE_BYTE | RTPHeaderExtensionFlags::TWO_BYTE
    }

    fn max_size(&self, _input: &gstreamer::BufferRef) -> usize {
        3
    }

    fn write(
        &self,
        _input: &gstreamer::BufferRef,
        _write_flags: RTPHeaderExtensionFlags,
        output: &mut gstreamer::BufferRef,
        output_data: &mut [u8],
    ) -> Result<usize, gstreamer::LoggableError> {
        assert!(output_data.len() >= 3);
        output_data[0..3].fill(0);
        core::result::Result::Ok(3)
    }

    fn read(
        &self,
        _read_flags: RTPHeaderExtensionFlags,
        input_data: &[u8],
        output: &mut gstreamer::BufferRef,
    ) -> Result<(), gstreamer::LoggableError> {
        assert_eq!(input_data.len(), 3); 
        core::result::Result::Ok(())
    }
    
    /*fn set_non_rtp_sink_caps(&self, caps: &gstreamer::Caps) -> Result<(), gstreamer::LoggableError> {
        core::result::Result::Ok(())
    }
    
    fn update_non_rtp_src_caps(&self, caps: &mut gstreamer::CapsRef) -> Result<(), gstreamer::LoggableError> {
        core::result::Result::Ok(())
    }
    
    fn set_attributes(
        &self,
        direction: gstreamer_rtp::RTPHeaderExtensionDirection,
        attributes: &str,
    ) -> Result<(), gstreamer::LoggableError> {
        core::result::Result::Ok(())
    }
    
    fn set_caps_from_attributes(&self, caps: &mut gstreamer::CapsRef) -> Result<(), gstreamer::LoggableError> {
        core::result::Result::Ok(())
    }*/
}

// https://gitlab.freedesktop.org/gstreamer/gstreamer/-/blob/main/subprojects/gst-examples/webrtc/multiparty-sendrecv/gst-rust/src/main.rs?ref_type=heads#L305
pub struct WebRTCPeer {
    pub id: String,
    pub queue: gstreamer::Element,
    pub webrtcbin: gstreamer::Element,
    pub may_offer: bool,
    pub bin: gstreamer::Bin,
    pub pad: gstreamer::GhostPad,
    pub data_channels: Vec<WebRTCDataChannel>,
}

impl WebRTCPeer {
    pub fn new(id: String) -> Self {
        let bin = gstreamer::Bin::new();
        let webrtcbin = gstreamer::ElementFactory::make("webrtcbin")
        .property_from_str("stun-server", "stun://stun.l.google.com:19302")
        .property_from_str("latency", "0")
        // .property_from_str("bundle-policy", "max-bundle")
        .build().expect("could not create webrtcbin element");
        let queue = gstreamer::ElementFactory::make("queue").property_from_str("leaky", "downstream").build().expect("could not create queue element");
        // this reduces freeze on low motion
        queue.set_property_from_str("max-size-time", "0");
        queue.set_property_from_str("max-size-bytes", "0");
        // may be temp disabled for debugging, this will work on local network anyways (at least i hope)
        // webrtcbin.set_property_from_str("stun-server", "stun://stun.l.google.com:19302");
        // webrtcbin.set_property_from_str("bundle-policy", "max-bundle");
        bin.add(&webrtcbin).expect("adding webrtcbin to bin failed");
        bin.add(&queue).expect("adding queue to bin failed");

        let input_pad = queue.static_pad("sink").expect("Could not get queue sink pad");
        let input_ghost_pad = GhostPad::with_target(&input_pad).expect("Could not create ghost pad");
        bin.add_pad(&input_ghost_pad).expect("Could not add ghost pad to queue element");

        Self {
            id,
            queue,
            webrtcbin,
            may_offer: true,
            bin: bin,
            pad: input_ghost_pad,
            data_channels: vec![],
        }
    }

    pub fn add_default_data_channels(&mut self) {
        // https://github.com/servo/media/blob/45756bef67037ade0f4f0125d579fdc3f3d457c8/backends/gstreamer/datachannel.rs#L17
        // TODO: dry this

        let unreliable_init_struct = gstreamer::Structure::builder("options")
            .field("ordered", false)
            .field("max-retransmits", 0)
            .field("negotiated", false)
            .build();

        let unreliable_channel = self.webrtcbin.emit_by_name::<WebRTCDataChannel>("create-data-channel", &[&"unreliable", &unreliable_init_struct]);

        self.add_data_channel(unreliable_channel);

        let reliable_init_struct = gstreamer::Structure::builder("options")
            .field("ordered", true)
            .field("max-retransmits", -1)
            .field("negotiated", false)
            .build();

        let reliable_channel = self.webrtcbin.emit_by_name::<WebRTCDataChannel>("create-data-channel", &[&"reliable", &reliable_init_struct]);

        self.add_data_channel(reliable_channel);

    }

    pub fn add_data_channel(&mut self, channel: WebRTCDataChannel) {
        self.data_channels.push(channel);
    }

    pub fn get_data_channels(&self) -> &Vec<WebRTCDataChannel> {
        &self.data_channels
    }

    pub fn play(&self) -> anyhow::Result<()> {
        self.bin.set_state(gstreamer::State::Playing)?;
        self.queue.set_state(gstreamer::State::Playing)?;
        self.webrtcbin.set_state(gstreamer::State::Playing)?;
        Ok(())
    }

    pub fn set_stun_server(&self, stun_server: &str) {
        self.webrtcbin.set_property_from_str("stun-server", &stun_server);
    }

    pub fn setup_with_pipeline(&self, pipeline: &gstreamer::Pipeline, tee: &gstreamer::Element) -> anyhow::Result<()> {
        self.add_to_pipeline(pipeline)?;
        self.link_with_pipeline(pipeline, tee)?;
        Ok(())
    }

    pub fn add_to_pipeline(&self, pipeline: &gstreamer::Pipeline)-> anyhow::Result<()> {
        // pipeline.add_many([&self.queue, &self.webrtcbin]).expect("adding elements to pipeline failed");
        pipeline.add(&self.queue)?;
        pipeline.add(&self.webrtcbin)?;
        Ok(())
    }

    pub fn link_with_pipeline(&self, pipeline: &gstreamer::Pipeline, tee: &gstreamer::Element) -> anyhow::Result<()> {
        gstreamer::Element::link_many([tee, &self.queue, &self.webrtcbin])?;
        Ok(())
    }

    pub fn link_internally(&self) -> anyhow::Result<()>{
        gstreamer::Element::link_many([&self.queue, &self.webrtcbin])?;
        Ok(())
    }

    pub fn destroy(&self, pipeline: &gstreamer::Pipeline, tee: &gstreamer::Element) -> anyhow::Result<()>{

        // prepare for unlinking
        // ???

        // unlink queue and webrtcbin
        gstreamer::Element::unlink_many([&tee, &self.queue, &self.webrtcbin]);

        // stop queue and webrtcbin
        self.stop()?;

        // remove queue and webrtcbin from pipeline
        self.remove_from_pipeline(pipeline)?;

        Ok(())
    }

    pub fn remove_from_pipeline(&self, pipeline: &gstreamer::Pipeline) -> anyhow::Result<()> {
        // pipeline.remove(&self.queue)?;
        // pipeline.remove(&self.webrtcbin)?;
        pipeline.remove(&self.bin)?;
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()>{
        self.queue.set_state(gstreamer::State::Null)?;
        self.webrtcbin.set_state(gstreamer::State::Null)?;
        Ok(())
    }

    pub fn set_local_description(&self, desc_ref: &WebRTCSessionDescription) {
        self.webrtcbin.emit_by_name::<()>("set-local-description", &[desc_ref, &None::<gstreamer::Promise>]);
    }

    pub fn set_remote_description(&self, desc_ref: &WebRTCSessionDescription) {
        self.webrtcbin.emit_by_name::<()>("set-remote-description", &[desc_ref, &None::<gstreamer::Promise>]);
    }

    pub fn set_remote_description_and_cb(&self, desc_ref: &WebRTCSessionDescription, on_promise: Box<dyn FnOnce() + Send>) {
        let promise = gstreamer::Promise::with_change_func(move |reply| {
            match reply {
                core::result::Result::Ok(_) => {
                    on_promise();
                },
                Err(err) => {
                    println!("failed at getting answer struct: {:?}", err);
                }
            }
        });
        self.webrtcbin.emit_by_name::<()>("set-remote-description", &[desc_ref, &promise]);
    }

    pub fn process_sdp_answer(&self, sdp: &str) -> anyhow::Result<()> {
        let sdp_message = gstreamer_sdp::SDPMessage::parse_buffer(sdp.as_bytes())?;
        let answer = gstreamer_webrtc::WebRTCSessionDescription::new(gstreamer_webrtc::WebRTCSDPType::Answer, sdp_message);
        self.set_remote_description(&answer);
        Ok(())
    }

    pub fn process_sdp_offer(&self, sdp: &str, on_answer: Box<dyn FnOnce(WebRTCSessionDescription) + Send>) -> anyhow::Result<()> {
        let sdp_message = gstreamer_sdp::SDPMessage::parse_buffer(sdp.as_bytes())?;
        let offer = gstreamer_webrtc::WebRTCSessionDescription::new(gstreamer_webrtc::WebRTCSDPType::Offer, sdp_message);
        // self.set_remote_description(&offer);
        let webrtc_ref = self.webrtcbin.clone();// we can do this lol thank god
        let promise = gstreamer::Promise::with_change_func(move |reply| {
            let answer_recv_promise = gstreamer::Promise::with_change_func(move |reply| {
                match reply {
                    core::result::Result::Ok(answer_option) => {
                        match answer_option {
                            Some(struct_ref) => {
                                let answer_result = struct_ref
                                    .value("answer")
                                    .unwrap()
                                    .get::<gstreamer_webrtc::WebRTCSessionDescription>();
                                match answer_result {
                                    core::result::Result::Ok(answer) => {
                                        on_answer(answer);
                                    },
                                    Err(err) => {
                                        println!("failed at getting answer struct: {:?}", err);
                                    }
                                }
                            },
                            None => {
                                println!("no answer created");
                            }
                        }
                    },
                    Err(err) => {
                        println!("early error creating answer: {:?}", err);
                    }
                }
            });
            webrtc_ref.emit_by_name::<()>("create-answer", &[&None::<gstreamer::Structure>, &answer_recv_promise]);
        });
        self.webrtcbin.emit_by_name::<()>("set-remote-description", &[&offer, &promise]);

        Ok(())
    }
}

pub struct WebRTCPreprocessor {
    pub encoder: gstreamer::Element,
    pub payloader: gstreamer::Element,
    pub extra_prefix_elements: Vec<gstreamer::Element>,
    pub extra_middle_elements: Vec<gstreamer::Element>,
    pub extra_suffix_elements: Vec<gstreamer::Element>,
    pub preset: EncodingPreset,
    settings: HashMap<String, serde_json::Value>, // this allows us to set the settings for the encoder and payloader regardless of format easily
    pub config: Option<Arc<StreamerConfig>>,
    pub pt: u32,
}

lazy_static! {
    pub static ref SUPPORTS_TARGET_BITRATE: Vec<EncodingPreset> = vec![EncodingPreset::H264, EncodingPreset::H265, EncodingPreset::VP8, EncodingPreset::VP9];
    pub static ref SUPPORTS_BITRATE: Vec<EncodingPreset> = vec![EncodingPreset::H264, EncodingPreset::H265, EncodingPreset::VP8, EncodingPreset::VP9, EncodingPreset::AV1];
    pub static ref SUPPORTS_DEADLINE: Vec<EncodingPreset> = vec![EncodingPreset::VP8, EncodingPreset::VP9];
}

// TODO: pass config to this and do hardware accel
// this is fast enough for now
// also AV1 asm encoder go brrr?
impl WebRTCPreprocessor {
    pub fn new(encoder: gstreamer::Element, payloader: gstreamer::Element) -> Self {
        Self {
            encoder,
            payloader,
            preset: EncodingPreset::Unknown,
            settings: HashMap::new(),
            extra_prefix_elements: Vec::new(),
            extra_middle_elements: Vec::new(),
            extra_suffix_elements: Vec::new(),
            config: None,
            pt: 96
        }
    }

    pub fn for_each_element<F>(&self, mut f: F)
    where
        F: FnMut(&gstreamer::Element),
    {
        for el in &self.extra_prefix_elements {
            f(el);
        }
        f(&self.encoder);
        for el in &self.extra_middle_elements {
            f(el);
        }
        f(&self.payloader);
        for el in &self.extra_suffix_elements {
            f(el);
        }
    }

    pub fn attach_to_pipeline(&self, pipeline: &gstreamer::Pipeline, after_element: &gstreamer::Element) {
        pipeline.add_many([&self.encoder, &self.payloader]).expect("adding elements to pipeline failed");
        // add prefix and suffix elements
        // I might just be able to add them directly with add_many but I'll check later
        pipeline.add_many(&self.extra_prefix_elements).expect("adding prefix elements to pipeline failed");
        pipeline.add_many(&self.extra_middle_elements).expect("adding middle elements to pipeline failed");
        pipeline.add_many(&self.extra_suffix_elements).expect("adding suffix elements to pipeline failed");

        let mut linkage = vec![after_element];
        for el in &self.extra_prefix_elements {
            linkage.push(el);
        }
        linkage.push(&self.encoder);
        for el in &self.extra_middle_elements {
            linkage.push(el);
        }
        linkage.push(&self.payloader);
        for el in &self.extra_suffix_elements {
            linkage.push(el);
        }

        gstreamer::Element::link_many(linkage).expect("linking elements failed");

        /*if let Some(parser_element) = pipeline.by_name("parser") {
            return;
            // relink with caps restriction
            println!("relinking parser element");
            match self.preset {
                EncodingPreset::H264 => {
                    self.encoder.unlink(&parser_element);
                    self.encoder.link_filtered(&parser_element, &gstreamer::Caps::builder("video/x-h264").field("stream-format", "byte-stream").field("profile", "constrained-baseline").build()).expect("linking elements failed");
                }
                EncodingPreset::H265 => {
                    self.encoder.unlink(&parser_element);
                    self.encoder.link_filtered(&parser_element, &gstreamer::Caps::builder("video/x-h265").field("stream-format", "byte-stream").field("profile", "constrained-baseline").build()).expect("linking elements failed");
                },
                _ => {

                }
            }
        }*/
    }

    pub fn get_last_element(&self) -> &gstreamer::Element {
        if self.extra_suffix_elements.len() > 0 {
            return &self.extra_suffix_elements[self.extra_suffix_elements.len() - 1];
        }
        &self.payloader
    }

    pub fn play(&self) -> anyhow::Result<()> {
        for el in &self.extra_prefix_elements {
            el.set_state(gstreamer::State::Playing)?;
        }
        self.encoder.set_state(gstreamer::State::Playing)?;
        for el in &self.extra_middle_elements {
            el.set_state(gstreamer::State::Playing)?;
        }
        self.payloader.set_state(gstreamer::State::Playing)?;
        for el in &self.extra_suffix_elements {
            el.set_state(gstreamer::State::Playing)?;
        }
        Ok(())
    }

    pub fn video_rtpcaps_partial_builder() -> gstreamer::caps::Builder<NoFeature> {
        // caps optimizations from https://github.com/selkies-project/selkies-gstreamer/blob/4583cf287d342f10189e986b1df1745cd75bb1ed/src/selkies_gstreamer/gstwebrtc_app.py#L838
        gstreamer::Caps::builder("application/x-rtp")
            .field("media", &"video")
            .field("rtcp-fb-nack-pli", &true)
            .field("rtcp-fb-ccm-fir", &true)
            .field("rtcp-fb-x-gstreamer-fir-as-repair", &true)
    }

    // maybe a better name for this is encodertype not preset
    pub fn new_preset(preset: EncodingPreset, optimizations: PipelineOptimization) -> Self {
        let mut prefix: Vec<gstreamer::Element> = vec![];
        let mut middle: Vec<gstreamer::Element> = vec![];
        let mut suffix: Vec<gstreamer::Element> = vec![];

        // TODO: support unknown
        let encoder_el_type = WebRTCPreprocessor::get_encoder_element_type(preset, optimizations);
        let payloader_el_type = WebRTCPreprocessor::get_payloader_element_type(preset, optimizations);

        println!("encoder type: {:?}", encoder_el_type);
        println!("payloader type: {:?}", payloader_el_type);

        match optimizations {
            PipelineOptimization::NVIDIA | PipelineOptimization::AMD => {
                // convert to NV12 format
                prefix.push(gstreamer::ElementFactory::make("videoconvert").name("nvvideoconverter").build().expect("Could not build nvidia videoconverter"));
                prefix.push(build_capsfilter(gstreamer::Caps::builder("video/x-raw").field("format", "NV12").build()).expect("could not create special capsfilter"));
            },
            _ => {

            }
        }

        let mut profile = "constrained-baseline";

        match optimizations {
            PipelineOptimization::NVIDIA => {

                

                match preset {
                    EncodingPreset::H264 => {
                        // prefix.push(gstreamer::ElementFactory::make("cudaupload").build().expect("could not create cudaupload element"));
                        // prefix.push(gstreamer::ElementFactory::make("cudaconvert").build().expect("could not create cudaconvert element"));
                        // prefix.push(build_capsfilter(gstreamer::Caps::builder("video/x-raw").features(["memory:CUDAMemory"]).field("format", "I420").build()).expect("could not create special capsfilter"));
                        // middle.push(build_capsfilter(gstreamer::Caps::builder("video/x-h264").field("stream-format", "byte-stream").field("profile", "constrained-baseline").build()).expect("could not create special capsfilter"));
                        // above is to use constrained baseline for compat
                        middle.push(build_capsfilter(gstreamer::Caps::builder("video/x-h264").field("stream-format", "byte-stream").field("profile", "main").build()).expect("could not create special capsfilter"));
                        profile = "main";
                        middle.push(gstreamer::ElementFactory::make("h264parse").name("parser").build().expect("could not create h264parse element"));
                        // great reference: https://github.com/m1k1o/neko/blob/21a4b2b797bb91947ed3702b8d26a99fef4ca157/server/internal/capture/pipelines.go#L158C40-L158C283
                        // video/x-h264,stream-format=byte-stream,profile=constrained-baseline
                        // middle.push(build_capsfilter(gstreamer::Caps::builder("video/x-h264").field("stream-format", "byte-stream").field("profile", "constrained-baseline").build()).expect("could not create special capsfilter"));
                    },
                    EncodingPreset::H265 => {
                        // middle.push(build_capsfilter(gstreamer::Caps::builder("video/x-h265").field("stream-format", "byte-stream").field("profile", "constrained-baseline").build()).expect("could not create special capsfilter"));
                        // above is to use constrained baseline for compat
                        middle.push(build_capsfilter(gstreamer::Caps::builder("video/x-h265").field("stream-format", "byte-stream").field("profile", "main").build()).expect("could not create special capsfilter"));
                        profile = "main";
                        middle.push(gstreamer::ElementFactory::make("h265parse").name("parser").build().expect("could not create h265parse element"));
                        // middle.push(build_capsfilter(gstreamer::Caps::builder("video/x-h265").field("stream-format", "byte-stream").field("profile", "main-444").build()).expect("could not create special capsfilter"));
                    },
                    _ => {

                    }
                }
            },
            PipelineOptimization::AMD => {
                match preset {
                    EncodingPreset::H264 => {
                        println!("pushing capsfilter for h264");
                        // middle.push(gstreamer::ElementFactory::make("queue").build().expect("could not create workaround queue element"));
                        // middle.push(build_capsfilter(gstreamer::Caps::builder("video/x-h264").field("stream-format", "byte-stream").field("profile", "main").build()).expect("could not create special capsfilter"));
                        middle.push(gstreamer::ElementFactory::make("h264parse").name("parser").build().expect("could not create h264parse element"));
                    },
                    EncodingPreset::H265 => {
                        println!("pushing capsfilter for h265");
                        // middle.push(gstreamer::ElementFactory::make("queue").build().expect("could not create workaround queue element"));
                        // middle.push(build_capsfilter(gstreamer::Caps::builder("video/x-h265").field("stream-format", "byte-stream").field("profile", "main").build()).expect("could not create special capsfilter"));
                        middle.push(gstreamer::ElementFactory::make("h265parse").name("parser").build().expect("could not create h265parse element"));
                    },
                    _ => {

                    }
                }
            }
            _ => {

            }
        }

        let mut pt = 96;

        match preset {
            // TODO: figure out more if I need to vary "payload"
            // https://github.com/selkies-project/selkies-gstreamer/blob/4583cf287d342f10189e986b1df1745cd75bb1ed/src/selkies_gstreamer/gstwebrtc_app.py#L836
            EncodingPreset::H264 => {
                let caps_builder = Self::video_rtpcaps_partial_builder();
                let caps = caps_builder.field("encoding-name", &"H264").field("payload", 97).build();
                pt = 97;
                // old: gstreamer::Caps::from_str("application/x-rtp,media=video,encoding-name=H264,payload=96").expect("Could not use default H264 caps");
                suffix.push(build_capsfilter(caps).expect("Could not construct rtp capsfilter"));
            },
            EncodingPreset::H265 => {
                let caps_builder = Self::video_rtpcaps_partial_builder();
                let caps = caps_builder.field("encoding-name", &"H265").field("payload", 100).build();
                pt = 100;
                // old: gstreamer::Caps::from_str("application/x-rtp,media=video,encoding-name=H265,payload=96").expect("Could not use default H265 caps");
                suffix.push(build_capsfilter(caps).expect("Could not construct rtp capsfilter"));
            },
            _ => {
                // don't put capsfilter yet
            }
        }

        Self {
            encoder: gstreamer::ElementFactory::make(&encoder_el_type).name("encoder").build().expect("could not create encoder element"),
            payloader: gstreamer::ElementFactory::make(&payloader_el_type).name("payloader").build().expect("could not create payloader element"),
            preset,
            settings: HashMap::new(),
            extra_prefix_elements: prefix,
            extra_middle_elements: middle,
            extra_suffix_elements: suffix,
            config: None,
            pt,
        }
    }

    pub fn set_config(&mut self, config: Arc<StreamerConfig>) {
        self.config = Some(config);
    }

    pub fn get_optimizations(&self) -> PipelineOptimization {
        match &self.config {
            Some(config) => {
                config.optimizations
            },
            None => PipelineOptimization::None,
        }
    }

    // TODO: support audio

    pub fn get_encoder_element_type_simple(preset: EncodingPreset) -> String {
        Self::get_encoder_element_type(preset, PipelineOptimization::None)
    }

    pub fn get_encoder_element_type(preset: EncodingPreset, optimizations: PipelineOptimization) -> String {
        match preset {
            // wow supermaven so smart???
            EncodingPreset::H264 => {
                match optimizations {
                    PipelineOptimization::NVIDIA => "nvh264enc".to_string(),
                    PipelineOptimization::Intel => "vah264enc".to_string(),
                    PipelineOptimization::AMD => "vah264enc".to_string(),
                    _ => "openh264enc".to_string(),
                }
            },
            EncodingPreset::H265 => {
                match optimizations {
                    PipelineOptimization::NVIDIA => "nvh265enc".to_string(),
                    PipelineOptimization::Intel => "vah265enc".to_string(),
                    PipelineOptimization::AMD => "vah265enc".to_string(),
                    _ => "x265enc".to_string(),
                }
            },
            EncodingPreset::VP8 => "vp8enc".to_string(),
            EncodingPreset::VP9 => "vp9enc".to_string(),
            EncodingPreset::AV1 => "rav1enc".to_string(),
            EncodingPreset::Unknown => panic!("unknown encoding preset should provide details on how to create the encoder"),
        }
    }

    pub fn get_payloader_element_type_simple(preset: EncodingPreset) -> String {
        Self::get_payloader_element_type(preset, PipelineOptimization::None)
    }

    pub fn get_payloader_element_type(preset: EncodingPreset, optimizations: PipelineOptimization) -> String {
        match preset {
            // wow supermaven so smart???
            EncodingPreset::H264 => "rtph264pay".to_string(),
            EncodingPreset::H265 => "rtph265pay".to_string(),
            EncodingPreset::VP8 => "rtpvp8pay".to_string(),
            EncodingPreset::VP9 => "rtpvp9pay".to_string(),
            EncodingPreset::AV1 => "rtpav1pay".to_string(), // https://gstreamer.freedesktop.org/documentation/rsrtp/rtpav1pay.html?gi-language=c
            EncodingPreset::Unknown => panic!("unknown encoding preset should provide details on how to create the payloader"),
        }
    }

    pub fn add_congestion_control_extension(&mut self){
        let twcc = gstreamer_rtp::RTPHeaderExtension::create_from_uri(TWCC_URI).unwrap();
        twcc.set_id(1);
        self.payloader.emit_by_name::<()>("add-extension", &[&twcc]);
    }

    // two funcs are heavily adapted from https://github.com/centricular/aes67-relay-chunker/blob/main/src/bin/fragment-enc.rs#L244
    // which saved me a lot of time figuring out how gstreamer works in rust
    pub fn install_experimental_extensions_globally() -> anyhow::Result<()> {
        gstreamer::Element::register(
            None,
            "x-playoutdelay",
            gstreamer::Rank::NONE,
            inner::PlayoutDelayRTPExperiment::static_type(),
        )?;
        Ok(())
    }

    pub fn add_experimental_extension(&mut self) -> anyhow::Result<()> {
        // TODO: make this actually catch errors
        let playout_delay = gstreamer::ElementFactory::make("x-playoutdelay")
            .build()
            .unwrap()
            .downcast::<gstreamer_rtp::RTPHeaderExtension>().expect("could not downcast to x-playoutdelay");
        playout_delay.set_id(2);
        self.payloader.emit_by_name::<()>("add-extension", &[&playout_delay]);
        Ok(())
    }

    pub fn set_default_settings(&mut self){
        self.payloader.set_property_from_str("config-interval", "-1");
        self.payloader.set_property_from_str("pt", "96");
        self.payloader.set_property_from_str("aggregate-mode", "zero-latency");

        match self.preset {
            // TODO: fix this, but it defaults to 0 which is based off resolution
            EncodingPreset::H264 => {
                match self.get_optimizations() {
                    PipelineOptimization::None => {
                        self.encoder.set_property_from_str("rate-control", "1");
                        self.set_setting("bitrate", serde_json::json!(1024 * 1024 * 6));
                        
                    },
                    PipelineOptimization::NVIDIA => {
                        self.encoder.set_property("zerolatency", true);
                        self.encoder.set_property_from_str("preset","low-latency-hp");
                        // parser
                        self.extra_middle_elements[1].set_property("config-interval", -1 as i32);
                        // https://github.com/m1k1o/neko/blob/master/server/internal/capture/pipelines.go
                        // preset=2 gop-size=25 spatial-aq=true temporal-aq=true bitrate=%d vbv-buffer-size=%d rc-mode=6
                        // self.encoder.set_property_from_str("rc-mode", "3");
                        /*self.encoder.set_property_from_str("rc-mode", "cbr");
                        self.encoder.set_property_from_str("rc-lookahead", "0");
                        self.encoder.set_property("b-adapt", false);
                        self.encoder.set_property("aud", false);
                        self.encoder.set_property_from_str("bitrate", "1024000");

                        // self.encoder.set_property_from_str("preset", "2");
                        self.encoder.set_property_from_str("gop-size", "60");
                        self.encoder.set_property_from_str("spatial-aq", "true");
                        self.encoder.set_property_from_str("temporal-aq", "true");*/
                        // self.encoder.set_property_from_str("bitrate", "1024000");
                        // self.encoder.set_property_from_str("vbv-buffer-size", "1024000");
                        
                    },
                    PipelineOptimization::AMD => {
                        // set for h264parse
                        self.extra_middle_elements[0].set_property("config-interval", -1 as i32);
                        self.encoder.set_property("aud", false);
                        self.encoder.set_property_from_str("target-usage", "4");
                    },
                    _ => {}
                }
            },
            EncodingPreset::H265 => {
                match self.get_optimizations() {
                    PipelineOptimization::None => {
                        self.set_setting("bitrate", serde_json::json!(1024 * 1024 * 4));
                    },
                    PipelineOptimization::AMD => {
                        self.extra_middle_elements[0].set_property("config-interval", -1 as i32);
                        self.encoder.set_property("aud", false);
                        self.encoder.set_property_from_str("target-usage", "6");
                    },
                    _ => {}
                }
                // self.set_setting("bitrate", serde_json::json!(1024 * 1024 * 4))
            },
            EncodingPreset::VP8 => self.set_setting("target-bitrate", serde_json::json!(1024 * 1024 * 6)),
            EncodingPreset::VP9 => self.set_setting("target-bitrate", serde_json::json!(1024 * 1024 * 4)),
            EncodingPreset::AV1 => self.set_setting("target-bitrate", serde_json::json!(1024 * 1024 * 4)),
            _ => {
                // idk
            }
        }

        if SUPPORTS_DEADLINE.contains(&self.preset) {
            self.set_setting("deadline", serde_json::json!(0));
        }
    }

    pub fn set_setting(&mut self, key: &str, value: serde_json::Value) {

        // match with preset
        let value_clone = value.clone();
        if matches!(self.preset, EncodingPreset::Unknown) {

        } else {
            match key {
                "target-bitrate" => {
                    if SUPPORTS_TARGET_BITRATE.contains(&self.preset) {
                        if let serde_json::Value::Number(value_number) = value {
                            // makes a uint in gstreamer strictness
                            let mut bitrate: i32 = value_number.as_u64().unwrap_or(1024 * 1024 * 4) as i32;
                            bitrate = num::clamp(bitrate, 1024 * 300, 1024 * 1024 * 100);
                            self.encoder.set_property("target-bitrate", bitrate); // gint
                        }
                    }
                },
                "bitrate" => {
                    if SUPPORTS_BITRATE.contains(&self.preset) {
                        if let serde_json::Value::Number(value_number) = value {
                            // makes a uint in gstreamer strictness
                            let mut bitrate: u32 = value_number.as_u64().unwrap_or(1024 * 1024 * 4) as u32;
                            bitrate = num::clamp(bitrate, 1024 * 300, 1024 * 1024 * 100);
                            self.encoder.set_property("bitrate", bitrate); //guint
                        }
                    }
                },
                "deadline" => {
                    if SUPPORTS_DEADLINE.contains(&self.preset) {
                        if let serde_json::Value::Number(value_number) = value {
                            // makes a uint in gstreamer strictness
                            let mut deadline: i64 = value_number.as_i64().unwrap_or(0);
                            deadline = num::clamp(deadline, 0,1000);
                            self.encoder.set_property("deadline", deadline); 
                        }
                    }
                },
                _ => {
                    // unknown
                }
            }
        }

        self.settings.insert(key.to_string(), value_clone);
    }
}