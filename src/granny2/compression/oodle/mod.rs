mod dictionary;
mod pair;
mod parameters;
mod weight_window;

use std::{error::Error, fmt::Display, io::Read};

use crate::granny2::compression::buffer::Buffer;

use {dictionary::Dictionary, pair::Pair, parameters::Parameters, weight_window::WeightWindow};

static SIZES: [usize; 4] = [128, 192, 256, 512];

#[derive(Debug)]
pub struct Oodle {
    numerator: u32,
    denominator: u32,
    next_denominator: u32,
}

impl Oodle {
    pub fn decompress<T: Read>(
        reader: &mut T,
        compressed_size: usize,
        decompressed_size: usize,
        stop_0: usize,
        stop_1: usize,
    ) -> Result<Vec<u8>, OodleError> {
        let compressed = {
            let mut buffer = vec![0; compressed_size + 4];
            reader.read_exact(&mut buffer[..compressed_size])?;
            buffer
        };
        let mut decompressed = Buffer::new(decompressed_size);

        let mut compressed_stream = compressed.as_slice();

        let parameters = (0..3)
            .map(|_| Parameters::parse(&mut compressed_stream))
            .collect::<Result<Vec<_>, _>>()?;
        log::trace!("{:#?}", parameters);

        let mut decoder = Self {
            numerator: u32::from(compressed_stream[0]) >> 1,
            denominator: 0x80,
            next_denominator: 0,
        };

        for (parameter, stop) in parameters.iter().zip([stop_0, stop_1, decompressed_size]) {
            let mut dictionary = Dictionary::from(parameter);

            while decompressed.position() < stop {
                let shift = decoder.decompress_block(
                    &mut compressed_stream,
                    &mut dictionary,
                    &mut decompressed,
                )?;
                decompressed.advance(shift);
            }
        }

        Ok(decompressed.inner())
    }

    fn decompress_block(
        &mut self,
        block: &mut &[u8],
        dictionary: &mut Dictionary,
        decompressed: &mut Buffer,
    ) -> Result<usize, OodleError> {
        let mut d1 =
            self.try_decompress_block(block, &mut dictionary.size_window[dictionary.backref_size]);
        if d1.index != 0xffff {
            let new_val = self.decode_commit(block, 65);
            dictionary.size_window[dictionary.backref_size].values[usize::from(d1.index)] = new_val;
            d1.value = new_val;
        }
        dictionary.backref_size = usize::from(d1.value);

        if dictionary.backref_size > 0 {
            let backref_size = if dictionary.backref_size < 61 {
                dictionary.backref_size + 1
            } else {
                SIZES[dictionary.backref_size - 61]
            };

            let Ok(decoded_size) = u32::try_from(dictionary.decoded_size) else {
                unreachable!("Decoded size must be smaller than u32");
            };
            let Ok(backref_range) = u16::try_from(dictionary.backref_value_max.min(decoded_size))
            else {
                unreachable!("Backref Range should be smaller than u16.");
            };

            let mut d3 = self.try_decompress_block(block, &mut dictionary.lowbit_window);
            if d3.index != 0xffff {
                let Ok(lowbit) = u16::try_from(dictionary.lowbit_value_max) else {
                    unreachable!("Lowbit Value Max should be smaller than u16.");
                };
                let new_val = self.decode_commit(block, lowbit);
                dictionary.lowbit_window.values[usize::from(d3.index)] = new_val;
                d3.value = new_val;
            }

            let mut d4 = self.try_decompress_block(block, &mut dictionary.highbit_window);
            if d4.index != 0xffff {
                let new_val = self.decode_commit(block, backref_range / 1024 + 1);
                dictionary.highbit_window.values[usize::from(d4.index)] = new_val;
                d4.value = new_val;
            }

            let mut d5 = self
                .try_decompress_block(block, &mut dictionary.midbit_window[usize::from(d4.value)]);
            if d5.index != 0xffff {
                let new_val = self.decode_commit(block, (backref_range / 4 + 1).min(256));
                dictionary.midbit_window[usize::from(d4.value)].values[usize::from(d5.index)] =
                    new_val;
                d5.value = new_val;
            }

            let backref_offset = usize::from((d4.value << 10) + (d5.value << 2) + d3.value + 1);
            dictionary.decoded_size += backref_size;

            decompressed.backref(backref_size, backref_offset);

            Ok(backref_size)
        } else {
            let i = 0;
            let mut d2 = self.try_decompress_block(block, &mut dictionary.decoded_window[i]);
            if d2.index != 0xffff {
                let Ok(decoded_max) = u16::try_from(dictionary.decoded_value_max) else {
                    unreachable!("Decoded Value Max should be smaller than u16.");
                };
                let new_val = self.decode_commit(block, decoded_max);
                dictionary.decoded_window[i].values[usize::from(d2.index)] = new_val;
                d2.value = new_val;
            }

            decompressed.push(d2.value.to_le_bytes()[1]);
            dictionary.decoded_size += 1;

            Ok(1)
        }
    }

    fn try_decompress_block(
        &mut self,
        block: &mut &[u8],
        weight_window: &mut WeightWindow,
    ) -> pair::Pair {
        if weight_window.weight_total >= weight_window.threshold_range_rebuild {
            if weight_window.threshold_range_rebuild >= weight_window.threshold_weight_rebuild {
                weight_window.rebuild_weights();
            }
            weight_window.rebuild_ranges();
        }

        let value = self.decode(block, 0x4000);
        let range = weight_window
            .ranges
            .iter()
            .position(|range| *range > value)
            .unwrap_or(weight_window.ranges.len())
            - 1;

        self.commit(
            0x4000,
            weight_window.ranges[range],
            weight_window.ranges[range + 1] - weight_window.ranges[range],
        );

        weight_window.weights[range] += 1;
        weight_window.weight_total += 1;

        if range == 0 {
            Pair {
                index: 0xffff,
                value: weight_window.values[range],
            }
        } else if weight_window.weights.len() >= weight_window.ranges.len()
            && self.decode_commit(block, 2) == 1
        {
            let Ok(len_diff) =
                u16::try_from(weight_window.weights.len() - weight_window.ranges.len() + 1)
            else {
                unreachable!("Length difference must be smaller than u16");
            };
            let index =
                weight_window.ranges.len() + usize::from(self.decode_commit(block, len_diff)) - 1;

            weight_window.weights[index] += 2;
            weight_window.weight_total += 2;

            Pair {
                index: 0xffff,
                value: weight_window.values[index],
            }
        } else {
            weight_window.values.push(0);
            weight_window.weights.push(2);
            weight_window.weight_total += 2;

            if weight_window.weights.len() == usize::from(weight_window.count_cap) {
                weight_window.weight_total -= std::mem::take(&mut weight_window.weights[0]);
            }

            let Ok(values_len) = u16::try_from(weight_window.values.len()) else {
                unreachable!("Values len should be smaller than u16.");
            };

            Pair {
                index: values_len - 1,
                value: 0,
            }
        }
    }

    fn decode(&mut self, stream: &mut &[u8], max: u16) -> u16 {
        while self.denominator <= 0x800000 {
            self.numerator <<= 8;
            self.numerator |= (u32::from(stream[0]) << 7) & 0x80;
            self.numerator |= (u32::from(stream[1]) >> 1) & 0x7f;
            *stream = &stream[1..];
            self.denominator <<= 8;
        }

        self.next_denominator = self.denominator / u32::from(max);
        let Ok(next) = u16::try_from(self.numerator / self.next_denominator) else {
            unreachable!("Next should be smaller than a u16.");
        };
        next.min(max - 1)
    }

    fn commit(&mut self, max: u16, val: u16, err: u16) {
        self.numerator -= self.next_denominator * u32::from(val);

        if val + err < max {
            self.denominator = self.next_denominator * u32::from(err);
        } else {
            self.denominator -= self.next_denominator * u32::from(val);
        }
    }

    fn decode_commit(&mut self, stream: &mut &[u8], max: u16) -> u16 {
        let val = self.decode(stream, max);
        self.commit(max, val, 1);
        val
    }
}

#[derive(Debug)]
pub enum OodleError {
    Decompress,
    Io,
}

impl Display for OodleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decompress => write!(f, "Failed to decompress."),
            Self::Io => write!(f, "Oodle failed to read compressed data."),
        }
    }
}

impl Error for OodleError {}

impl From<std::io::Error> for OodleError {
    fn from(value: std::io::Error) -> Self {
        log::error!("{}", value);
        Self::Io
    }
}
