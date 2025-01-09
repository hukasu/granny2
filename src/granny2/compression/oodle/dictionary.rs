use std::borrow::Borrow;

use super::{parameters::Parameters, weight_window::WeightWindow};

#[derive(Debug)]
pub struct Dictionary {
    pub decoded_size: usize,
    pub backref_size: usize,

    pub decoded_value_max: u32,
    pub backref_value_max: u32,
    pub lowbit_value_max: u32,
    pub midbit_value_max: u32,
    pub highbit_value_max: u32,

    pub lowbit_window: WeightWindow,
    pub highbit_window: WeightWindow,
    pub midbit_window: Vec<WeightWindow>,

    pub decoded_window: Vec<WeightWindow>,
    pub size_window: Vec<WeightWindow>,
}

impl<T> From<T> for Dictionary
where
    T: Borrow<super::Parameters>,
{
    fn from(value: T) -> Self {
        let parameters: &Parameters = value.borrow();

        let lowbit_value_max = (parameters.backref_value_max + 1).min(4);
        let midbit_value_max = ((parameters.backref_value_max / 4) + 1).min(256);
        let highbit_value_max = (parameters.backref_value_max / 1024) + 1;

        let Ok(lowbit_value_max_16) = u16::try_from(lowbit_value_max) else {
            unreachable!("Lowbit Value Max must be smaller than u16.");
        };
        let Ok(midbit_value_max_u16) = u16::try_from(midbit_value_max) else {
            unreachable!("Midbit Value Max must be smaller than u16.");
        };
        let Ok(decoded_count_u16) = u16::try_from(parameters.decoded_count) else {
            unreachable!("Decoded Count Max must be smaller than u16.");
        };
        let Ok(highbit_count) = u16::try_from(parameters.highbit_count) else {
            unreachable!("Highbit Value Max must be smaller than u16.");
        };
        Self {
            decoded_size: 0,
            backref_size: 0,
            decoded_value_max: parameters.decoded_value_max,
            backref_value_max: parameters.backref_value_max,
            lowbit_value_max,
            midbit_value_max,
            highbit_value_max,
            lowbit_window: WeightWindow::new(lowbit_value_max - 1, lowbit_value_max_16),
            midbit_window: (0..highbit_value_max)
                .map(|_| WeightWindow::new(midbit_value_max - 1, midbit_value_max_u16))
                .collect(),
            highbit_window: WeightWindow::new(highbit_value_max - 1, highbit_count + 1),
            decoded_window: (0..4)
                .map(move |_| {
                    WeightWindow::new(parameters.decoded_value_max - 1, decoded_count_u16)
                })
                .collect(),
            size_window: (0..4)
                .flat_map(|i| {
                    (0..16).map(move |_| {
                        WeightWindow::new(64, u16::from(parameters.sizes_count[3 - i]))
                    })
                })
                .chain([WeightWindow::new(64, u16::from(parameters.sizes_count[0]))])
                .collect(),
        }
    }
}
