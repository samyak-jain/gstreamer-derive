use gstreamer_derive::GStreamer;

#[derive(GStreamer)]
#[link(Src, URIDecodeBin, CudaUpload)]
enum GStreamerInput {
    #[name = "source"]
    Src,
    URIDecodeBin,
    CudaUpload,
}

fn main() {
    gstreamer::init().unwrap();
    let _ = GStreamerInput::build();
}
