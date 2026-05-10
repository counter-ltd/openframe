use crate::{Bounds, DisplayId, Pixels, PlatformDisplay, Point, px};
use anyhow::Result;

#[derive(Debug)]
pub(crate) struct IosDisplay {
    id: DisplayId,
    uuid: uuid::Uuid,
    bounds: Bounds<Pixels>,
}

impl IosDisplay {
    pub(crate) fn new() -> Self {
        Self {
            id: DisplayId(1),
            uuid: uuid::Uuid::new_v4(),
            bounds: Bounds::from_corners(Point::default(), Point::new(px(390.), px(844.))),
        }
    }
}

impl PlatformDisplay for IosDisplay {
    fn id(&self) -> crate::DisplayId {
        self.id
    }

    fn uuid(&self) -> Result<uuid::Uuid> {
        Ok(self.uuid)
    }

    fn bounds(&self) -> crate::Bounds<crate::Pixels> {
        self.bounds
    }
}
