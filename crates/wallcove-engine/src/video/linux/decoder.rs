use std::path::Path;
use std::str::FromStr;
use std::sync::mpsc::SyncSender;

use anyhow::{Context, anyhow};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_gl as gst_gl;
use gstreamer_gl::prelude::*;
use gstreamer_gl_egl as gst_gl_egl;
use gstreamer_video as gst_video;

const BUS_POLL_MS: u64 = 100;

pub struct Frame {
    pub texture_id: u32,
    pub width: u32,
    pub height: u32,
}

pub fn run_decoder(
    source: &Path,
    gl_display: gst_gl::GLDisplay,
    gl_context: gst_gl::GLContext,
    tx: SyncSender<gst::Sample>,
    mut should_stop: impl FnMut() -> bool,
) -> anyhow::Result<()> {
    let uri = path_to_uri(source)?;

    loop {
        if should_stop() {
            return Ok(());
        }

        let pipeline = build_pipeline(&uri).context("build decoder pipeline")?;

        let mut display_ctx = gst::Context::new("gst.gl.GLDisplay", true);
        display_ctx
            .get_mut()
            .expect("freshly created Context has a unique reference")
            .set_gl_display(&gl_display);

        pipeline.set_context(&display_ctx);

        let mut app_ctx = gst::Context::new("gst.gl.app_context", true);
        app_ctx
            .get_mut()
            .expect("freshly created Context has a unique reference")
            .structure_mut()
            .set("context", &gl_context);

        pipeline.set_context(&app_ctx);

        let appsink = pipeline
            .by_name("sink")
            .context("appsink not found in pipeline")?
            .downcast::<gst_app::AppSink>()
            .map_err(|_| anyhow!("`sink` is not an AppSink"))?;

        let tx_cb = tx.clone();
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    tx_cb.send(sample).map_err(|_| gst::FlowError::Eos)?;
                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        pipeline
            .set_state(gst::State::Playing)
            .context("set pipeline state to Playing")?;

        let bus = pipeline.bus().context("pipeline has no bus")?;
        let mut restart = false;

        loop {
            if should_stop() {
                pipeline.set_state(gst::State::Null).ok();
                return Ok(());
            }

            let msg = bus.timed_pop(gst::ClockTime::from_mseconds(BUS_POLL_MS));

            let Some(msg) = msg else {
                continue;
            };

            use gst::MessageView::*;
            match msg.view() {
                Eos(_) => {
                    if pipeline
                        .seek_simple(
                            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                            gst::ClockTime::ZERO,
                        )
                        .is_ok()
                    {
                        continue;
                    }
                    restart = true;
                    break;
                }
                Error(err) => {
                    pipeline.set_state(gst::State::Null).ok();
                    return Err(anyhow!(
                        "pipeline error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    ));
                }
                _ => {}
            }
        }

        pipeline.set_state(gst::State::Null).ok();

        if !restart {
            return Ok(());
        }
    }
}

pub fn sample_to_frame(
    sample: gst::Sample,
    gl_context: &gst_gl::GLContext,
) -> anyhow::Result<Frame> {
    let buffer = sample.buffer_owned().context("sample has no buffer")?;
    let caps = sample.caps().context("sample has no caps")?;
    let info = gst_video::VideoInfo::from_caps(caps).context("VideoInfo from caps")?;

    if let Some(sync_meta) = buffer.meta::<gst_gl::GLSyncMeta>() {
        sync_meta.wait(gl_context);
    }

    let gl_frame = gst_gl::GLVideoFrame::from_buffer_readable(buffer, &info)
        .map_err(|_| anyhow!("GLVideoFrame::from_buffer_readable failed"))?;

    let texture_id = gl_frame
        .texture_id(0)
        .context("GLVideoFrame texture id missing")?;

    Ok(Frame {
        texture_id,
        width: info.width(),
        height: info.height(),
    })
}

pub fn wrap_gl(
    egl_display: usize,
    egl_context: usize,
) -> anyhow::Result<(gst_gl::GLDisplay, gst_gl::GLContext)> {
    gst::init().context("gst::init")?;

    let gl_display = unsafe { gst_gl_egl::GLDisplayEGL::with_egl_display(egl_display) }
        .context("wrap EGL display for GStreamer")?;
    let gl_display = gl_display.upcast::<gst_gl::GLDisplay>();

    let gl_context = unsafe {
        gst_gl::GLContext::new_wrapped(
            &gl_display,
            egl_context,
            gst_gl::GLPlatform::EGL,
            gst_gl::GLAPI::GLES2,
        )
    }
    .context("wrap EGL context for GStreamer")?;

    gl_context
        .activate(true)
        .map_err(|_| anyhow!("GLContext::activate failed"))?;

    gl_context
        .fill_info()
        .map_err(|e| anyhow!("GLContext::fill_info: {e}"))?;

    Ok((gl_display, gl_context))
}

fn path_to_uri(source: &Path) -> anyhow::Result<String> {
    let abs_path = source
        .canonicalize()
        .with_context(|| format!("resolve path {}", source.display()))?;
    Ok(gst::glib::filename_to_uri(abs_path, None)
        .context("convert path to file:// URI")?
        .to_string())
}

fn build_pipeline(uri: &str) -> anyhow::Result<gst::Pipeline> {
    let pipeline = gst::Pipeline::default();

    let decodebin = gst::ElementFactory::make("uridecodebin3")
        .property("uri", uri)
        .build()
        .context("create uridecodebin3")?;

    let glupload = gst::ElementFactory::make("glupload")
        .build()
        .context("create glupload")?;
    let glcolorconvert = gst::ElementFactory::make("glcolorconvert")
        .build()
        .context("create glcolorconvert")?;

    let sink_caps =
        gst::Caps::from_str("video/x-raw(memory:GLMemory),format=RGBA,texture-target=2D")
            .context("parse appsink caps")?;
    let appsink = gst_app::AppSink::builder()
        .name("sink")
        .caps(&sink_caps)
        .sync(true)
        .max_buffers(1)
        .build();

    pipeline
        .add_many([&decodebin, &glupload, &glcolorconvert, appsink.upcast_ref()])
        .context("add elements to pipeline")?;

    gst::Element::link_many([&glupload, &glcolorconvert, appsink.upcast_ref()])
        .context("link glupload -> glcolorconvert -> appsink")?;

    let glupload_weak = glupload.downgrade();
    decodebin.connect_pad_added(move |_, src_pad| {
        let Some(glupload) = glupload_weak.upgrade() else {
            return;
        };
        let Some(sink_pad) = glupload.static_pad("sink") else {
            return;
        };
        if sink_pad.is_linked() {
            return;
        }
        if let Err(err) = src_pad.link(&sink_pad) {
            tracing::warn!("link uridecodebin3 video pad -> glupload: {err:?}");
        }
    });

    Ok(pipeline)
}
