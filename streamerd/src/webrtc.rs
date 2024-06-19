use gstreamer::{prelude::*, Buffer, BufferFlags, Element, ErrorMessage};
use gstreamer_video::{prelude::*, VideoInfo};

pub struct WebRTCPeer {
    pub id: String,
    pub queue: gstreamer::Element,
    pub webrtcbin: gstreamer::Element,
}

impl WebRTCPeer {
    pub fn new(id: String) -> Self {
        Self {
            id,
            queue: gstreamer::ElementFactory::make("queue").build().expect("could not create queue element"),
            webrtcbin: gstreamer::ElementFactory::make("webrtcbin").build().expect("could not create webrtcbin element"),
        }
    }

    pub fn setup_with_pipeline(&self, pipeline: &gstreamer::Pipeline, tee: &gstreamer::Element) {
        self.add_to_pipeline(pipeline);
        self.link_with_pipeline(pipeline, tee);
    }

    pub fn add_to_pipeline(&self, pipeline: &gstreamer::Pipeline) {
        pipeline.add_many([&self.queue, &self.webrtcbin]).expect("adding elements to pipeline failed");
    }

    pub fn link_with_pipeline(&self, pipeline: &gstreamer::Pipeline, tee: &gstreamer::Element) {
        gstreamer::Element::link_many([tee, &self.queue, &self.webrtcbin]).expect("linking elements failed");
    }
}