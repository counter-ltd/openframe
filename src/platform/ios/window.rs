use crate::{
    AnyWindowHandle, AtlasKey, AtlasTextureId, AtlasTile, Bounds, DispatchEventResult, GpuSpecs,
    Pixels, PlatformAtlas, PlatformDisplay, PlatformInput, PlatformInputHandler, PlatformWindow,
    Point, PromptButton, RequestFrameOptions, Size, TileId, WindowAppearance,
    WindowBackgroundAppearance, WindowBounds, WindowControlArea, WindowParams,
};
use collections::HashMap;
use parking_lot::Mutex;
use raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle};
use std::{
    cell::Cell,
    rc::{Rc, Weak},
    sync::{self, Arc},
};

use super::IosPlatform;

pub(crate) struct IosWindowState {
    pub(crate) bounds: Bounds<Pixels>,
    pub(crate) handle: AnyWindowHandle,
    display: Rc<dyn PlatformDisplay>,
    pub(crate) title: Option<String>,
    pub(crate) edited: bool,
    platform: Weak<IosPlatform>,
    sprite_atlas: Arc<dyn PlatformAtlas>,
    pub(crate) should_close_handler: Option<Box<dyn FnMut() -> bool>>,
    hit_test_window_control_callback: Option<Box<dyn FnMut() -> Option<WindowControlArea>>>,
    input_callback: Option<Box<dyn FnMut(PlatformInput) -> DispatchEventResult>>,
    active_status_change_callback: Option<Box<dyn FnMut(bool)>>,
    hover_status_change_callback: Option<Box<dyn FnMut(bool)>>,
    resize_callback: Option<Box<dyn FnMut(Size<Pixels>, f32)>>,
    moved_callback: Option<Box<dyn FnMut()>>,
    input_handler: Option<PlatformInputHandler>,
    is_fullscreen: bool,
    /// Host-owned `CAMetalLayer` pointer (opaque `usize`), set from Swift when embedding.
    pub(crate) host_metal_layer: Cell<Option<usize>>,
}

#[derive(Clone)]
pub(crate) struct IosWindow(pub(crate) Rc<Mutex<IosWindowState>>);

impl HasWindowHandle for IosWindow {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl HasDisplayHandle for IosWindow {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl IosWindow {
    pub(crate) fn new(
        handle: AnyWindowHandle,
        params: WindowParams,
        platform: Weak<IosPlatform>,
        display: Rc<dyn PlatformDisplay>,
    ) -> Self {
        Self(Rc::new(Mutex::new(IosWindowState {
            bounds: params.bounds,
            display,
            platform,
            handle,
            sprite_atlas: Arc::new(IosAtlas::new()),
            title: Default::default(),
            edited: false,
            should_close_handler: None,
            hit_test_window_control_callback: None,
            input_callback: None,
            active_status_change_callback: None,
            hover_status_change_callback: None,
            resize_callback: None,
            moved_callback: None,
            input_handler: None,
            is_fullscreen: false,
            host_metal_layer: Cell::new(None),
        })))
    }

    pub(crate) fn handle(&self) -> AnyWindowHandle {
        self.0.lock().handle
    }

    /// Bridge [`UITouch`] / UIKit events into GPUI ([`PlatformInput`] uses mouse events on iOS).
    pub fn inject_platform_input(&self, event: PlatformInput) -> bool {
        let mut lock = self.0.lock();
        let Some(mut callback) = lock.input_callback.take() else {
            return false;
        };
        drop(lock);
        let result = callback(event);
        self.0.lock().input_callback = Some(callback);
        !result.propagate
    }

    /// Store an opaque pointer to the host view’s `CAMetalLayer` for Metal presentation.
    pub fn set_host_metal_layer_ptr(&self, ptr: usize) {
        self.0.lock().host_metal_layer.set(Some(ptr));
    }
}

impl PlatformWindow for IosWindow {
    fn bounds(&self) -> Bounds<Pixels> {
        self.0.lock().bounds
    }

    fn window_bounds(&self) -> WindowBounds {
        WindowBounds::Windowed(self.bounds())
    }

    fn is_maximized(&self) -> bool {
        false
    }

    fn content_size(&self) -> Size<Pixels> {
        self.bounds().size
    }

    fn resize(&mut self, size: Size<Pixels>) {
        self.0.lock().bounds.size = size;
    }

    fn scale_factor(&self) -> f32 {
        3.0
    }

    fn appearance(&self) -> WindowAppearance {
        WindowAppearance::Light
    }

    fn display(&self) -> Option<Rc<dyn crate::PlatformDisplay>> {
        Some(self.0.lock().display.clone())
    }

    fn mouse_position(&self) -> Point<Pixels> {
        Point::default()
    }

    fn modifiers(&self) -> crate::Modifiers {
        crate::Modifiers::default()
    }

    fn capslock(&self) -> crate::Capslock {
        crate::Capslock::default()
    }

    fn set_input_handler(&mut self, input_handler: PlatformInputHandler) {
        self.0.lock().input_handler = Some(input_handler);
    }

    fn take_input_handler(&mut self) -> Option<PlatformInputHandler> {
        self.0.lock().input_handler.take()
    }

    fn prompt(
        &self,
        _level: crate::PromptLevel,
        _msg: &str,
        _detail: Option<&str>,
        _answers: &[PromptButton],
    ) -> Option<futures::channel::oneshot::Receiver<usize>> {
        let (tx, rx) = futures::channel::oneshot::channel();
        tx.send(0).ok();
        Some(rx)
    }

    fn activate(&self) {}

    fn is_active(&self) -> bool {
        true
    }

    fn is_hovered(&self) -> bool {
        true
    }

    fn set_title(&mut self, title: &str) {
        self.0.lock().title = Some(title.to_owned());
    }

    fn set_app_id(&mut self, _app_id: &str) {}

    fn set_background_appearance(&self, _background: WindowBackgroundAppearance) {}

    fn set_edited(&mut self, edited: bool) {
        self.0.lock().edited = edited;
    }

    fn show_character_palette(&self) {}

    fn minimize(&self) {}

    fn zoom(&self) {}

    fn toggle_fullscreen(&self) {
        let mut lock = self.0.lock();
        lock.is_fullscreen = !lock.is_fullscreen;
    }

    fn is_fullscreen(&self) -> bool {
        self.0.lock().is_fullscreen
    }

    fn on_request_frame(&self, _callback: Box<dyn FnMut(RequestFrameOptions)>) {}

    fn on_input(&self, callback: Box<dyn FnMut(crate::PlatformInput) -> DispatchEventResult>) {
        self.0.lock().input_callback = Some(callback)
    }

    fn on_active_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.0.lock().active_status_change_callback = Some(callback)
    }

    fn on_hover_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.0.lock().hover_status_change_callback = Some(callback)
    }

    fn on_resize(&self, callback: Box<dyn FnMut(Size<Pixels>, f32)>) {
        self.0.lock().resize_callback = Some(callback)
    }

    fn on_moved(&self, callback: Box<dyn FnMut()>) {
        self.0.lock().moved_callback = Some(callback)
    }

    fn on_should_close(&self, callback: Box<dyn FnMut() -> bool>) {
        self.0.lock().should_close_handler = Some(callback);
    }

    fn on_close(&self, _callback: Box<dyn FnOnce()>) {}

    fn on_hit_test_window_control(&self, callback: Box<dyn FnMut() -> Option<WindowControlArea>>) {
        self.0.lock().hit_test_window_control_callback = Some(callback);
    }

    fn on_appearance_changed(&self, _callback: Box<dyn FnMut()>) {}

    fn draw(&self, scene: &crate::Scene) {
        let _ = (scene, super::metal_renderer::SHADERS_METALLIB.len());
    }

    fn sprite_atlas(&self) -> sync::Arc<dyn crate::PlatformAtlas> {
        self.0.lock().sprite_atlas.clone()
    }

    #[cfg(any(test, feature = "test-support"))]
    fn as_test(&mut self) -> Option<&mut crate::TestWindow> {
        None
    }

    #[cfg(target_os = "windows")]
    fn get_raw_handle(&self) -> windows::Win32::Foundation::HWND {
        unimplemented!()
    }

    fn show_window_menu(&self, _position: Point<Pixels>) {}

    fn start_window_move(&self) {}

    fn update_ime_position(&self, _bounds: Bounds<Pixels>) {}

    fn gpu_specs(&self) -> Option<GpuSpecs> {
        metal::Device::system_default().map(|device| GpuSpecs {
            is_software_emulated: false,
            device_name: device.name().to_string(),
            driver_name: "Metal".into(),
            driver_info: String::new(),
        })
    }
}

pub(crate) struct IosAtlasState {
    next_id: u32,
    tiles: HashMap<AtlasKey, AtlasTile>,
}

pub(crate) struct IosAtlas(Mutex<IosAtlasState>);

impl IosAtlas {
    pub(crate) fn new() -> Self {
        IosAtlas(Mutex::new(IosAtlasState {
            next_id: 0,
            tiles: HashMap::default(),
        }))
    }
}

impl PlatformAtlas for IosAtlas {
    fn get_or_insert_with<'a>(
        &self,
        key: &crate::AtlasKey,
        build: &mut dyn FnMut() -> anyhow::Result<
            Option<(Size<crate::DevicePixels>, std::borrow::Cow<'a, [u8]>)>,
        >,
    ) -> anyhow::Result<Option<crate::AtlasTile>> {
        let mut state = self.0.lock();
        if let Some(tile) = state.tiles.get(key) {
            return Ok(Some(tile.clone()));
        }
        drop(state);

        let Some((size, _)) = build()? else {
            return Ok(None);
        };

        let mut state = self.0.lock();
        state.next_id += 1;
        let texture_id = state.next_id;
        state.next_id += 1;
        let tile_id = state.next_id;

        state.tiles.insert(
            key.clone(),
            crate::AtlasTile {
                texture_id: AtlasTextureId {
                    index: texture_id,
                    kind: crate::AtlasTextureKind::Monochrome,
                },
                tile_id: TileId(tile_id),
                padding: 0,
                bounds: crate::Bounds {
                    origin: Point::default(),
                    size,
                },
            },
        );

        Ok(Some(state.tiles[key].clone()))
    }

    fn remove(&self, key: &AtlasKey) {
        let mut state = self.0.lock();
        state.tiles.remove(key);
    }
}
