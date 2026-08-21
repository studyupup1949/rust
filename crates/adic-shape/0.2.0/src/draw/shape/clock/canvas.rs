use bon::bon;
use std::iter::repeat;
use adic::{
    error::AdicError,
    traits::{CanTruncate, HasApproximateDigits},
};

use crate::{
    error::{AdicShapeError, AdicShapeResult},
    shape::{canvas_sealed, AdicCanvas},
};
use super::shape::{ClockHand, ClockMovement, ClockShape};


#[derive(Debug, Clone)]
/// Canvas for drawing [`ClockShape`]
pub struct ClockCanvas {
    base: u32,
    depth: isize,
    clock_movement: ClockMovement,
    show_tick_labels: bool,
    show_val_circles: bool,
    viewbox_width: u32,
    viewbox_height: u32,
}



#[bon]
impl ClockCanvas {

    #[builder]
    /// Start [`ClockCanvasBuilder`] to build a `ClockCanvas`
    pub fn new(
        /// Number of ticks on the clock
        base: u32,
        /// Maximum clock hand valuation, the depth of the clock
        depth: isize,
        #[builder(default = ClockMovement::Sweeping)]
        /// Ticking or sweeping clock hands
        clock_movement: ClockMovement,
        #[builder(default = true)]
        /// Enable to label tick marks
        show_tick_labels: bool,
        #[builder(default = false)]
        /// Enable to show dotted lines at the radius of the clock hand heads
        show_val_circles: bool,
        #[builder(default = 100)]
        /// Width of the clock window
        viewbox_width: u32,
        #[builder(default = 100)]
        /// Height of the clock window
        viewbox_height: u32,
    ) -> Self {

        Self {
            base,
            depth,
            clock_movement,
            show_tick_labels,
            show_val_circles,
            viewbox_width,
            viewbox_height,
        }

    }

}

impl ClockCanvas {

    /// Number of branches from each branch point
    pub fn base(&self) -> u32 {
        self.base
    }

}

impl From<ClockCanvas> for AdicShapeResult<ClockCanvas> {
    fn from(value: ClockCanvas) -> Self {
        Ok(value)
    }
}


impl AdicCanvas for ClockCanvas {
    type Shape = ClockShape;
}

impl canvas_sealed::DrawSingleInteger for ClockCanvas {
    fn _draw_integer(
        &self,
        adic_integer: &(impl Clone + HasApproximateDigits<DigitIndex = usize>),
    ) -> AdicShapeResult<ClockShape> {

        if u32::from(adic_integer.base()) != self.base() {
            Err(AdicShapeError::AdicError(AdicError::MixedCharacteristic))?;
        }

        let min_valuation = 0;
        let max_valuation = self.depth;
        let depth = usize::try_from(self.depth)?;
        if adic_integer.certainty().finite().is_some_and(|c| c < depth) {
            Err(AdicShapeError::AdicError(AdicError::InappropriatePrecision("Integer is not precise enough to draw on clock".to_string())))?;
        }

        let mut hands = Vec::with_capacity(depth);
        let mut offset = 0.0;
        for (i, d) in adic_integer.digits().chain(repeat(0)).take(depth).enumerate() {
            hands.push(ClockHand{
                order: i.try_into()?,
                tick: d,
                offset,
            });
            offset = match self.clock_movement {
                ClockMovement::Ticking => 0.0,
                ClockMovement::Sweeping => {
                    (f64::from(d) + offset) / f64::from(self.base)
                }
            };
        }

        let shape = ClockShape::new(
            self.base,
            hands,
            min_valuation,
            max_valuation,
            self.show_tick_labels,
            self.show_val_circles,
            false,
            self.viewbox_width,
            self.viewbox_height,
        );

        Ok(shape)

    }
}

impl canvas_sealed::DrawSingleNumber for ClockCanvas {
    fn _draw_number(
        &self,
        adic_number: &(impl Clone + HasApproximateDigits<DigitIndex = isize> + CanTruncate),
    ) -> AdicShapeResult<ClockShape> {

        if u32::from(adic_number.base()) != self.base() {
            Err(AdicShapeError::AdicError(AdicError::MixedCharacteristic))?;
        }

        if adic_number.certainty().finite().is_some_and(|c| c < self.depth) {
            Err(AdicShapeError::AdicError(AdicError::InappropriatePrecision("Number is not precise enough to draw on clock".to_string())))?;
        }

        let min_valuation = adic_number.min_index().finite().map_or(0, |v| if v < 0 { v } else { 0 });
        let max_valuation = self.depth;
        let num_hands = (max_valuation - min_valuation).try_into()?;
        let mut hands = Vec::with_capacity(num_hands);
        let mut offset = 0.0;
        for (i, d) in adic_number.digits().chain(repeat(0)).take(num_hands).enumerate() {
            hands.push(ClockHand{
                order: i.try_into()?,
                tick: d,
                offset,
            });
            offset = match self.clock_movement {
                ClockMovement::Ticking => 0.0,
                ClockMovement::Sweeping => {
                    (f64::from(d) + offset) / f64::from(self.base)
                }
            };
        }

        let shape = ClockShape::new(
            self.base,
            hands,
            min_valuation,
            max_valuation,
            self.show_tick_labels,
            self.show_val_circles,
            true,
            self.viewbox_width,
            self.viewbox_height,
        );

        Ok(shape)

    }
}
