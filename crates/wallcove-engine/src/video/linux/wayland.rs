use std::collections::HashMap;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

use anyhow::{Context, bail};
use gstreamer as gst;
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, WEnum, delegate_noop,
    protocol::{wl_callback, wl_compositor, wl_output, wl_region, wl_registry, wl_surface},
};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use super::decoder::{self, Frame};
use super::renderer::{GlRenderer, ScaleMode};

pub struct WaylandState {
    conn: Connection,
    compositor: Option<wl_compositor::WlCompositor>,
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    outputs: HashMap<u32, OutputState>,
}

struct OutputState {
    wl_output: wl_output::WlOutput,
    name: String,
    width: u32,
    height: u32,
    output_done: bool,
    wl_surface: Option<wl_surface::WlSurface>,
    layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    configured: bool,
    frame_callback_pending: bool,
    registry_name: u32,
}

impl WaylandState {
    fn new(conn: Connection) -> Self {
        Self {
            conn,
            compositor: None,
            layer_shell: None,
            outputs: HashMap::new(),
        }
    }

    fn any_unconfigured(&self, names: &mut Vec<u32>) -> bool {
        names.retain(|n| self.outputs.contains_key(n));
        names
            .iter()
            .any(|n| self.outputs.get(n).is_some_and(|o| !o.configured))
    }

    fn wait_for_frame_callbacks(
        &mut self,
        eq: &mut EventQueue<Self>,
        shutdown: &Receiver<()>,
    ) -> anyhow::Result<bool> {
        let deadline = std::time::Instant::now() + Duration::from_millis(500);

        while self.outputs.values().any(|o| o.frame_callback_pending) {
            if shutdown.try_recv().is_ok() {
                self.cancel_pending_frame_callbacks();
                return Ok(true);
            }

            if std::time::Instant::now() >= deadline {
                tracing::warn!("frame callback timed out during shutdown wait; continuing");
                self.cancel_pending_frame_callbacks();
                return Ok(true);
            }

            eq.blocking_dispatch(self)
                .context("waiting for frame callback")?;
        }

        Ok(false)
    }

    fn cancel_pending_frame_callbacks(&mut self) {
        for output in self.outputs.values_mut() {
            output.frame_callback_pending = false;
        }
    }

    fn teardown_surfaces(&mut self, eq: &mut EventQueue<Self>) {
        tracing::info!("destroying wayland layer surfaces");
        self.cancel_pending_frame_callbacks();

        for output in self.outputs.values_mut() {
            if let Some(layer) = output.layer_surface.take() {
                layer.destroy();
            }
            output.wl_surface = None;
            output.configured = false;
        }

        let _ = eq.dispatch_pending(self);
        let _ = self.conn.flush();
        let _ = eq.roundtrip(self);
    }

    fn create_layer_surface(
        &mut self,
        registry_name: u32,
        compositor: &wl_compositor::WlCompositor,
        layer_shell: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        qh: &QueueHandle<Self>,
    ) {
        let output = self.outputs.get_mut(&registry_name).expect("output exists");

        let wl_surface = compositor.create_surface(qh, ());

        // Empty input region so pointer events reach the Plasma desktop/icons beneath.
        let empty_region = compositor.create_region(qh, ());
        wl_surface.set_input_region(Some(&empty_region));

        let layer_surface = layer_shell.get_layer_surface(
            &wl_surface,
            Some(&output.wl_output),
            zwlr_layer_shell_v1::Layer::Background,
            "wallcove".to_string(),
            qh,
            registry_name,
        );
        layer_surface.set_size(0, 0);
        layer_surface.set_anchor(zwlr_layer_surface_v1::Anchor::all());
        layer_surface.set_exclusive_zone(-1);
        layer_surface.set_keyboard_interactivity(
            zwlr_layer_surface_v1::KeyboardInteractivity::None,
        );

        wl_surface.commit();

        output.wl_surface = Some(wl_surface);
        output.layer_surface = Some(layer_surface);
    }
}

pub fn run_session(
    video_path: std::path::PathBuf,
    shutdown: Receiver<()>,
    ready: SyncSender<Result<(), String>>,
    stop_flag: Arc<AtomicBool>,
    exited: SyncSender<()>,
) {
    let result = run_session_inner(video_path, shutdown, &ready, stop_flag);
    if let Err(err) = result {
        tracing::error!(error = %err, "wayland video session failed");
        ready.send(Err(err.to_string())).ok();
    }
    let _ = exited.send(());
}

fn run_session_inner(
    video_path: std::path::PathBuf,
    shutdown: Receiver<()>,
    ready: &SyncSender<Result<(), String>>,
    stop_flag: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let conn = Connection::connect_to_env().context("connect to Wayland display")?;
    let mut eq = conn.new_event_queue();
    let qh = eq.handle();
    let mut state = WaylandState::new(conn);
    state.conn.display().get_registry(&qh, ());
    eq.roundtrip(&mut state).context("registry roundtrip")?;
    eq.roundtrip(&mut state).context("output info roundtrip")?;

    let compositor = state
        .compositor
        .clone()
        .context("wl_compositor missing")?;
    let layer_shell = state
        .layer_shell
        .clone()
        .context(
            "zwlr_layer_shell_v1 missing: compositor does not support layer-shell wallpapers",
        )?;

    let mut chosen: Vec<u32> = state
        .outputs
        .iter()
        .filter(|(_, o)| o.output_done)
        .map(|(name, _)| *name)
        .collect();
    chosen.sort_by_key(|name| state.outputs[name].name.clone());

    if chosen.is_empty() {
        bail!("no Wayland outputs found");
    }

    // Prototype: mirror to the first configured output.
    let primary = chosen[0];
    let mut outputs = vec![primary];

    for &name in &outputs {
        state.create_layer_surface(name, &compositor, &layer_shell, &qh);
    }

    while state.any_unconfigured(&mut outputs) {
        if shutdown.try_recv().is_ok() {
            state.teardown_surfaces(&mut eq);
            return Ok(());
        }
        eq.blocking_dispatch(&mut state)
            .context("waiting for layer surface configure")?;
    }

    let primary_output = state.outputs.get(&primary).context("primary output missing")?;
    let width = primary_output.width.max(1);
    let height = primary_output.height.max(1);
    let wl_surface = primary_output.wl_surface.clone().context("missing wl_surface")?;

    let mut renderer =
        GlRenderer::new(&state.conn, &wl_surface, width, height).context("create GL renderer")?;

    let (gl_display, gl_context) =
        decoder::wrap_gl(renderer.egl_display(), renderer.egl_context())?;

    let (frame_tx, frame_rx) = std::sync::mpsc::sync_channel::<gst::Sample>(1);
    let decoder_stop = Arc::clone(&stop_flag);
    let decoder_path = video_path.clone();
    let decoder_gl_context = gl_context.clone();

    let decoder_thread = thread::Builder::new()
        .name("wallcove-decoder".into())
        .spawn(move || {
            let _ = decoder::run_decoder(
                &decoder_path,
                gl_display,
                decoder_gl_context,
                frame_tx,
                || decoder_stop.load(Ordering::Relaxed),
            );
        })
        .context("spawn decoder thread")?;

    let mut signaled_ready = false;
    let mut shutting_down = false;

    loop {
        if shutdown.try_recv().is_ok() {
            shutting_down = true;
            break;
        }

        if state.wait_for_frame_callbacks(&mut eq, &shutdown)? {
            shutting_down = true;
            break;
        }

        let sample = match frame_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(sample) => sample,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        };

        if shutdown.try_recv().is_ok() {
            shutting_down = true;
            break;
        }

        let frame = decoder::sample_to_frame(sample, &gl_context)
            .context("convert decoded sample to GL frame")?;

        if !signaled_ready {
            ready
                .send(Ok(()))
                .map_err(|_| anyhow::anyhow!("video ready signal receiver dropped"))?;
            signaled_ready = true;
        }

        render_frame(
            &mut state,
            &mut eq,
            &qh,
            &mut renderer,
            primary,
            &frame,
        )?;

        eq.dispatch_pending(&mut state)
            .context("dispatch pending Wayland events")?;
        state.conn.flush().context("flush Wayland connection")?;
    }

    stop_flag.store(true, Ordering::Relaxed);
    state.teardown_surfaces(&mut eq);
    drop(renderer);

    let decoder_join = thread::Builder::new()
        .name("wallcove-decoder-join".into())
        .spawn(move || {
            let _ = decoder_thread.join();
        });

    if let Ok(join_handle) = decoder_join {
        if join_handle
            .join()
            .is_err()
        {
            tracing::warn!("decoder join thread panicked");
        }
    }

    if shutting_down {
        tracing::info!("wayland video session stopped");
    }

    Ok(())
}

fn render_frame(
    state: &mut WaylandState,
    eq: &mut EventQueue<WaylandState>,
    qh: &QueueHandle<WaylandState>,
    renderer: &mut GlRenderer,
    output_name: u32,
    frame: &Frame,
) -> anyhow::Result<()> {
    let output = state.outputs.get_mut(&output_name).context("output missing")?;
    let wl_surface = output.wl_surface.as_ref().context("missing wl_surface")?;
    wl_surface.frame(qh, output_name);
    output.frame_callback_pending = true;

    renderer.render(0, frame, ScaleMode::Fill)?;

    eq.dispatch_pending(state).context("dispatch after render")?;
    Ok(())
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(registry.bind(name, version.min(4), qh, ()));
                }
                "zwlr_layer_shell_v1" => {
                    state.layer_shell = Some(registry.bind(name, version.min(4), qh, ()));
                }
                "wl_output" => {
                    let output: wl_output::WlOutput = registry.bind(name, version.min(4), qh, name);
                    state.outputs.insert(
                        name,
                        OutputState {
                            wl_output: output,
                            name: String::new(),
                            width: 1,
                            height: 1,
                            output_done: false,
                            wl_surface: None,
                            layer_surface: None,
                            configured: false,
                            frame_callback_pending: false,
                            registry_name: name,
                        },
                    );
                }
                _ => {}
            },
            wl_registry::Event::GlobalRemove { name } => {
                state.outputs.remove(&name);
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, u32> for WaylandState {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        registry_name: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(output) = state.outputs.get_mut(registry_name) else {
            return;
        };

        match event {
            wl_output::Event::Name { name } => output.name = name,
            wl_output::Event::Mode {
                flags,
                width,
                height,
                ..
            } => {
                let is_current = matches!(
                    flags,
                    WEnum::Value(mode) if mode.contains(wl_output::Mode::Current)
                );
                if is_current {
                    output.width = width.max(0) as u32;
                    output.height = height.max(0) as u32;
                }
            }
            wl_output::Event::Done => output.output_done = true,
            _ => {}
        }
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, u32> for WaylandState {
    fn event(
        state: &mut Self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        registry_name: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                layer_surface.ack_configure(serial);
                if let Some(output) = state.outputs.get_mut(registry_name) {
                    if width > 0 && height > 0 {
                        output.width = width as u32;
                        output.height = height as u32;
                    }
                    output.configured = true;
                }
            }
            zwlr_layer_surface_v1::Event::Closed => {
                state.outputs.remove(registry_name);
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_callback::WlCallback, u32> for WaylandState {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        registry_name: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            if let Some(output) = state.outputs.get_mut(registry_name) {
                output.frame_callback_pending = false;
            }
        }
    }
}

delegate_noop!(WaylandState: ignore wl_compositor::WlCompositor);
delegate_noop!(WaylandState: ignore wl_surface::WlSurface);
delegate_noop!(WaylandState: ignore wl_region::WlRegion);
delegate_noop!(WaylandState: ignore zwlr_layer_shell_v1::ZwlrLayerShellV1);
