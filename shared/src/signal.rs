//! Signal shaping functions

/// Heartbeats, looking at high-to-low sensor transitions.
/// Fast rate of change, less likely to confuse with noise
/// (flip of the sign of the derivative).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Heartbeat {
    pub high_idx: usize,
    pub high_value: f32,
    pub low_idx: usize,
    pub low_value: f32,
}

impl Heartbeat {
    pub fn zero() -> Self {
        Heartbeat {
            high_idx: 0,
            high_value: 0.0,
            low_idx: 0,
            low_value: 0.0,
        }
    }
}

pub struct HeartbeatItr<'a, const N: usize> {
    deriv_itr: DerivItr<'a, N>,
    high: Option<DerivItrItem>,
    last_deriv: Option<f32>,
}

impl<'a, const N: usize> HeartbeatItr<'a, N> {
    pub fn new(data: &'a [f32; N]) -> Self {
        HeartbeatItr {
            deriv_itr: DerivItr::new(data),
            high: None,
            last_deriv: None,
        }
    }
}

impl<'a, const N: usize> Iterator for HeartbeatItr<'a, N> {
    type Item = Heartbeat;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let d = self.deriv_itr.next()?;

            let last_deriv = match self.last_deriv {
                Some(ld) => ld,
                None => {
                    self.last_deriv = Some(d.deriv);
                    continue;
                }
            };

            self.last_deriv = Some(d.deriv);

            if d.deriv >= 0.0 {
                if last_deriv >= 0.0 {
                    self.high = Some(d);
                } else {
                    match self.high {
                        Some(h) => {
                            break Some(Heartbeat {
                                high_idx: h.idx,
                                high_value: h.sample,
                                low_idx: d.idx,
                                low_value: d.sample,
                            })
                        }
                        None => continue,
                    }
                }
            }
        }
    }
}

/// Sample, its index and a first derivative
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DerivItrItem {
    idx: usize,
    sample: f32,
    deriv: f32,
}

pub struct DerivItr<'a, const N: usize> {
    data: &'a [f32; N],
    idx: usize,
}

impl<'a, const N: usize> DerivItr<'a, N> {
    pub fn new(data: &'a [f32; N]) -> Self {
        DerivItr { data, idx: 0 }
    }
}

impl<'a, const N: usize> Iterator for DerivItr<'a, N> {
    type Item = DerivItrItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= N - 1 {
            None
        } else {
            let i = self.idx;
            self.idx += 1;

            let s = self.data[i];
            let s1 = self.data[i + 1];
            Some(DerivItrItem {
                idx: i,
                sample: s,
                deriv: s1 - s,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deriv_itr() {
        let data = [1.0, 1.0, 1.0];
        let mut ditr = DerivItr::new(&data);
        assert_eq!(
            ditr.next(),
            Some(DerivItrItem {
                idx: 0,
                sample: 1.0,
                deriv: 0.0
            })
        );
        assert_eq!(
            ditr.next(),
            Some(DerivItrItem {
                idx: 1,
                sample: 1.0,
                deriv: 0.0
            })
        );

        let data = [1.0, 2.0, 1.0];
        let mut ditr = DerivItr::new(&data);
        assert_eq!(
            ditr.next(),
            Some(DerivItrItem {
                idx: 0,
                sample: 1.0,
                deriv: 1.0
            })
        );
        assert_eq!(
            ditr.next(),
            Some(DerivItrItem {
                idx: 1,
                sample: 2.0,
                deriv: -1.0
            })
        );
    }

    #[test]
    fn test_heartbeat_itr() {
        let data = [
            1.0, 2.0, 3.0, 4.0, 5.0, 4.0, 3.0, 4.0, 5.0, 6.0, 4.0, 2.0, 0.0, 1.0,
        ];

        let mut hbitr = HeartbeatItr::new(&data);
        assert_eq!(
            hbitr.next(),
            Some(Heartbeat {
                high_idx: 3,
                high_value: 4.0,
                low_idx: 6,
                low_value: 3.0
            })
        );

        assert_eq!(
            hbitr.next(),
            Some(Heartbeat {
                high_idx: 8,
                high_value: 5.0,
                low_idx: 12,
                low_value: 0.0
            })
        );
    }
}
