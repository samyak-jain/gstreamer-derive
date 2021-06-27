enum GStreamerInput {
    #[name = "source"]
    Src,
}
use gstreamer::Element;
struct GStreamerGStreamerInput {
    pub Src: Element,
}
impl GStreamerInput {
    fn build() -> GStreamerGStreamerInput {
        let Src = gstreamer::ElementFactory::make(&"Src".to_lowercase(), Some("source")).unwrap();
        GStreamerGStreamerInput { Src }
    }
}
fn main() {
    gstreamer::init().unwrap();
    let test = GStreamerInput::build();
}
