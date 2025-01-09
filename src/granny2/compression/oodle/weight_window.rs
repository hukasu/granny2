#[derive(Debug)]
pub struct WeightWindow {
    pub count_cap: u16,

    pub ranges: Vec<u16>,
    pub values: Vec<u16>,
    pub weights: Vec<u16>,
    pub weight_total: u16,

    pub threshold_increase: u16,
    pub threshold_increase_cap: u16,
    pub threshold_range_rebuild: u16,
    pub threshold_weight_rebuild: u16,
}

impl WeightWindow {
    pub fn new(max_value: u32, count_cap: u16) -> Self {
        let threshold_weight_rebuild = 256.max(15160.min(32 * max_value));
        let threshold_increase_cap = if max_value > 64 {
            let Ok(cap) = u16::try_from((max_value * 2).min(threshold_weight_rebuild / 2 - 32))
            else {
                unreachable!("Threshold Increase Cap must be smaller than u16.");
            };
            cap
        } else {
            128
        };
        let Ok(threshold_weight_rebuild) = u16::try_from(threshold_weight_rebuild) else {
            unreachable!("Threshold Weight Rebuild must be smaller than u16.");
        };
        Self {
            count_cap: count_cap + 1,
            ranges: vec![0, 0x4000],
            values: vec![0],
            weights: vec![4],
            weight_total: 4,
            threshold_increase: 4,
            threshold_increase_cap,
            threshold_range_rebuild: 8,
            threshold_weight_rebuild,
        }
    }

    pub fn rebuild_ranges(&mut self) {
        if self.ranges.len() != self.weights.len() + 1 {
            assert!(self.ranges.len() > self.weights.len() + 1);
            self.ranges.truncate(self.weights.len() + 1);
        }

        let Ok(range_weight) = u16::try_from((8 * 0x4000) / u32::from(self.weight_total)) else {
            unreachable!("Range Weight must be smaller than u16.");
        };
        let mut range_start = 0;
        for (range, weight) in (self.ranges.iter_mut()).zip(self.weights.iter()) {
            *range = range_start;
            let Ok(next_range_start) =
                u16::try_from(u32::from(*weight) * u32::from(range_weight) / 8)
            else {
                unreachable!("Range start must be smaller than u16.");
            };
            range_start += next_range_start;
        }
        let len = self.ranges.len();
        self.ranges[len - 1] = 0x4000;

        if self.threshold_increase > self.threshold_increase_cap / 2 {
            self.threshold_range_rebuild = self.weight_total + self.threshold_increase_cap;
        } else {
            self.threshold_increase *= 2;
            self.threshold_range_rebuild = self.weight_total + self.threshold_increase;
        }
    }

    pub fn rebuild_weights(&mut self) {
        let mut weight_total = 0u16;
        self.weights.iter_mut().for_each(|weight| {
            *weight /= 2;
            weight_total += *weight;
        });
        self.weight_total = weight_total;

        let mut i = 1;
        while i < self.weights.len() {
            while i < self.weights.len() && self.weights[i] == 0 {
                self.weights.swap_remove(i);
                self.values.swap_remove(1);
            }
            i += 1;
        }

        if self.weights.len() > 1 {
            let Some(max) = self.weights[1..].iter().max() else {
                unreachable!("Weights must have more than 2 values");
            };
            if let Some(position) = self.weights[1..].iter().rposition(|weight| weight == max) {
                let len = self.weights.len();
                self.weights.swap(position, len - 1);
                self.values.swap(position, len - 1);
            }
        }

        if self.weights.len() < usize::from(self.count_cap) && self.weights[0] == 0 {
            self.weights[0] = 1;
            self.weight_total += 1;
        }
    }
}
