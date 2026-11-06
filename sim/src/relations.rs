//! Precise physical words. Rain, shelter, and examine all ask these
//! questions; they do not keep a second copy of the answer.

use crate::model::{CellId, ConstructionId, ItemId, Role, World};

impl World {
    pub(crate) fn exposure_millis(&self, cell: CellId) -> u32 {
        if self.cell(cell).rock {
            return self.tuning.rock_roof;
        }
        if let Some(frame) = self.construction_at(cell) {
            if frame.bark.is_some() && self.supported_at_both_ends(frame.id) {
                let moisture = frame
                    .bark
                    .and_then(|id| self.item(id))
                    .map(|item| item.moisture)
                    .unwrap_or(0);
                if moisture >= self.tuning.damp_at {
                    return self.tuning.wet_bark_roof;
                }
                return self.tuning.dry_bark_roof;
            }
        }
        1000
    }

    pub(crate) fn supported_at_both_ends(&self, id: ConstructionId) -> bool {
        let Some(frame) = self.constructions.get(&id) else {
            return false;
        };
        let (Some(left), Some(right), Some(beam)) = (frame.support_a, frame.support_b, frame.beam)
        else {
            return false;
        };
        self.placed_together(left, beam) && self.placed_together(right, beam)
    }

    pub(crate) fn placed_together(&self, a: ItemId, b: ItemId) -> bool {
        match (self.item(a), self.item(b)) {
            (Some(left), Some(right)) => {
                left.cell == right.cell && left.carrier.is_none() && right.carrier.is_none()
            }
            _ => false,
        }
    }

    pub(crate) fn item_exposure(&self, item_id: ItemId) -> u32 {
        let Some(item) = self.item(item_id) else {
            return 1000;
        };
        if item.role == Role::BarkCover {
            return 1000;
        }
        if let Some(carrier) = item.carrier {
            let cell = self.actor(carrier).map(|a| a.cell).unwrap_or(item.cell);
            return self.exposure_millis(cell);
        }
        self.exposure_millis(item.cell)
    }

    pub(crate) fn cover_phrase(&self, cell: CellId) -> &'static str {
        if self.cell(cell).rock {
            "under the rock overhang"
        } else if self
            .construction_at(cell)
            .map(|frame| frame.bark.is_some() && self.supported_at_both_ends(frame.id))
            .unwrap_or(false)
        {
            "under a bark roof"
        } else if self.raining {
            "in the rain"
        } else {
            "in the open"
        }
    }

    pub(crate) fn rain_reaches_phrase(&self, cell: CellId) -> &'static str {
        let exposure = self.exposure_millis(cell);
        if exposure == 0 {
            "Rain does not reach it."
        } else if exposure < 1000 {
            "Only a little rain reaches it."
        } else if self.raining {
            "Rain reaches it."
        } else {
            "The sky is open above it."
        }
    }
}
