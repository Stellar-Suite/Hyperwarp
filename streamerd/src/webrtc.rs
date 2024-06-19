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

    pub fn remove_from_pipeline(&self, pipeline: &gstreamer::Pipeline, tee: &gstreamer::Element) {

        // unlink queue and webrtcbin
        gstreamer::Element::unlink_many([&tee, &self.queue, &self.webrtcbin]);

        // stop queue and webrtcbin
        self.queue.set_state(gstreamer::State::Null).expect("could not set queue state to null");
        self.webrtcbin.set_state(gstreamer::State::Null).expect("could not set webrtcbin state to null");

        // remove queue and webrtcbin from pipeline
        pipeline.remove(&self.queue).expect("removing queue element failed");
        pipeline.remove(&self.webrtcbin).expect("removing webrtcbin element failed");
    }
}