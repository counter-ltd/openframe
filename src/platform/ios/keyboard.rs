use crate::PlatformKeyboardLayout;

pub(crate) struct IosKeyboardLayout;

impl PlatformKeyboardLayout for IosKeyboardLayout {
    fn id(&self) -> &str {
        "openframe.keyboard.ios"
    }

    fn name(&self) -> &str {
        "iOS Software Keyboard"
    }
}
