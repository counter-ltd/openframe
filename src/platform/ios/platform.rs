use super::{
    clipboard,
    IosDisplay,
    IosKeyboardLayout,
    IosWindow,
    schedule_application_launch,
};
use crate::{
    Action, AnyWindowHandle, ClipboardItem, CursorStyle, DummyKeyboardMapper,
    Keymap, Menu, MenuItem, NoopTextSystem, PathPromptOptions, Platform, PlatformDisplay,
    PlatformKeyboardLayout, PlatformKeyboardMapper, PlatformTextSystem, Task,
    WindowAppearance, WindowParams,
};
#[cfg(feature = "screen-capture")]
use crate::ScreenCaptureSource;
use anyhow::{Result, anyhow};
use futures::channel::oneshot;
#[cfg(feature = "screen-capture")]
use std::sync::Arc;
use parking_lot::Mutex;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    sync::Arc,
};

pub(crate) struct IosPlatform {
    background_executor: crate::BackgroundExecutor,
    foreground_executor: crate::ForegroundExecutor,
    text_system: Arc<dyn PlatformTextSystem>,
    active_display: Rc<dyn PlatformDisplay>,
    active_window: RefCell<Option<IosWindow>>,
    active_cursor: Mutex<CursorStyle>,
    opened_url: RefCell<Option<String>>,
    weak: Weak<Self>,
}

impl IosPlatform {
    pub(crate) fn new(_headless: bool) -> Rc<Self> {
        let dispatcher = Arc::new(super::IosDispatcher);
        let text_system = Arc::new(NoopTextSystem::new());

        Rc::new_cyclic(|weak| Self {
            background_executor: crate::BackgroundExecutor::new(dispatcher.clone()),
            foreground_executor: crate::ForegroundExecutor::new(dispatcher),
            text_system,
            active_display: Rc::new(IosDisplay::new()),
            active_window: RefCell::new(None),
            active_cursor: Mutex::new(CursorStyle::default()),
            opened_url: RefCell::new(None),
            weak: weak.clone(),
        })
    }
}

impl Platform for IosPlatform {
    fn background_executor(&self) -> crate::BackgroundExecutor {
        self.background_executor.clone()
    }

    fn foreground_executor(&self) -> crate::ForegroundExecutor {
        self.foreground_executor.clone()
    }

    fn text_system(&self) -> Arc<dyn PlatformTextSystem> {
        self.text_system.clone()
    }

    fn keyboard_layout(&self) -> Box<dyn PlatformKeyboardLayout> {
        Box::new(IosKeyboardLayout)
    }

    fn keyboard_mapper(&self) -> Rc<dyn PlatformKeyboardMapper> {
        Rc::new(DummyKeyboardMapper)
    }

    fn on_keyboard_layout_change(&self, _: Box<dyn FnMut()>) {}

    fn run(&self, on_finish_launching: Box<dyn FnOnce()>) {
        schedule_application_launch(on_finish_launching);
    }

    fn quit(&self) {}

    fn restart(&self, _: Option<PathBuf>) {}

    fn activate(&self, _: bool) {}

    fn hide(&self) {}

    fn hide_other_apps(&self) {}

    fn unhide_other_apps(&self) {}

    fn displays(&self) -> Vec<Rc<dyn PlatformDisplay>> {
        vec![self.active_display.clone()]
    }

    fn primary_display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        Some(self.active_display.clone())
    }

    #[cfg(feature = "screen-capture")]
    fn is_screen_capture_supported(&self) -> bool {
        false
    }

    #[cfg(feature = "screen-capture")]
    fn screen_capture_sources(
        &self,
    ) -> oneshot::Receiver<anyhow::Result<Vec<Arc<dyn ScreenCaptureSource>>>> {
        let (tx, rx) = oneshot::channel();
        tx.send(Ok(Vec::new())).ok();
        rx
    }

    fn active_window(&self) -> Option<AnyWindowHandle> {
        self.active_window
            .borrow()
            .as_ref()
            .map(|w| w.handle())
    }

    fn open_window(
        &self,
        handle: AnyWindowHandle,
        params: WindowParams,
    ) -> anyhow::Result<Box<dyn crate::PlatformWindow>> {
        let window = IosWindow::new(
            handle,
            params,
            self.weak.clone(),
            self.active_display.clone(),
        );
        *self.active_window.borrow_mut() = Some(window.clone());
        super::bridge::ios_store_active_window(window.clone());
        Ok(Box::new(window))
    }

    fn window_appearance(&self) -> WindowAppearance {
        WindowAppearance::Light
    }

    fn open_url(&self, url: &str) {
        *self.opened_url.borrow_mut() = Some(url.to_string());
    }

    fn on_open_urls(&self, _callback: Box<dyn FnMut(Vec<String>)>) {}

    fn prompt_for_paths(
        &self,
        _options: PathPromptOptions,
    ) -> oneshot::Receiver<Result<Option<Vec<PathBuf>>>> {
        let (tx, rx) = oneshot::channel();
        tx.send(Err(anyhow!("document picker not wired on iOS")))
            .ok();
        rx
    }

    fn prompt_for_new_path(
        &self,
        _directory: &Path,
        _suggested_name: Option<&str>,
    ) -> oneshot::Receiver<Result<Option<PathBuf>>> {
        let (tx, rx) = oneshot::channel();
        tx.send(Err(anyhow!("save panel not wired on iOS"))).ok();
        rx
    }

    fn can_select_mixed_files_and_dirs(&self) -> bool {
        false
    }

    fn reveal_path(&self, _path: &Path) {}

    fn open_with_system(&self, _path: &Path) {}

    fn on_quit(&self, _callback: Box<dyn FnMut()>) {}

    fn on_reopen(&self, _callback: Box<dyn FnMut()>) {}

    fn set_menus(&self, _menus: Vec<Menu>, _keymap: &Keymap) {}

    fn set_dock_menu(&self, _menu: Vec<MenuItem>, _keymap: &Keymap) {}

    fn add_recent_document(&self, _path: &Path) {}

    fn on_app_menu_action(&self, _callback: Box<dyn FnMut(&dyn Action)>) {}

    fn on_will_open_app_menu(&self, _callback: Box<dyn FnMut()>) {}

    fn on_validate_app_menu_command(&self, _callback: Box<dyn FnMut(&dyn Action) -> bool>) {}

    fn app_path(&self) -> Result<PathBuf> {
        clipboard::main_bundle_path()
    }

    fn path_for_auxiliary_executable(&self, _name: &str) -> Result<PathBuf> {
        Err(anyhow!("path_for_auxiliary_executable not implemented on iOS"))
    }

    fn set_cursor_style(&self, style: CursorStyle) {
        *self.active_cursor.lock() = style;
    }

    fn should_auto_hide_scrollbars(&self) -> bool {
        true
    }

    fn write_to_clipboard(&self, item: ClipboardItem) {
        clipboard::write_to_uikit_pasteboard(&item);
    }

    fn read_from_clipboard(&self) -> Option<ClipboardItem> {
        clipboard::read_from_uikit_pasteboard()
    }

    fn write_credentials(&self, _url: &str, _username: &str, _password: &[u8]) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }

    fn read_credentials(&self, _url: &str) -> Task<Result<Option<(String, Vec<u8>)>>> {
        Task::ready(Ok(None))
    }

    fn delete_credentials(&self, _url: &str) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }

    fn register_url_scheme(&self, _url: &str) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }
}
