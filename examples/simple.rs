use gstreamer_derive::GStreamer;

#[derive(GStreamer)]
#[link_elements(Src, URIDecodeBin, CudaUpload)]
enum GStreamerInput {
    #[name = "source"]
    Src,

    #[property(location = "test", uri = "hello")]
    URIDecodeBin,
    CudaUpload,
}

fn main() {
    gstreamer::init().unwrap();
    let _ = GStreamerInput::build();
}
