///! Formula for linear regression equation is given by:
///!  y=a+bx
///! a and b are given by the following formulas:
///! a(intercept)= (∑y∑x^2–∑x∑xy) / (n(∑x^2)–(∑x)^2)
///! b(slope)= (n∑xy−(∑x)(∑y)) / (n∑x^2−(∑x)^2)
///!
///! Where,
///! x and y are two variables on the regression line.
///! b = Slope of the line.
///! a = y-intercept of the line.
///! x = Values of the first data set.
///! y = Values of the second data set.

pub struct Linreg<const NUM_SAMPLES: usize> {
    pub intercept: f32,
    pub slope: f32,

    // a few constants that depend on the array size
    sumx: f32,
    sum_xsq: f32,
    sumx_sq: f32,
}

impl<const NUM_SAMPLES: usize> Linreg<NUM_SAMPLES> {
    pub fn new() -> Self {
        let sumx = ((NUM_SAMPLES - 1) * NUM_SAMPLES) as f32 / 2.0;
        Linreg {
            intercept: 0.0,
            slope: 1.0,
            sumx,
            sum_xsq: (0..NUM_SAMPLES).map(|x| x as f32 * x as f32).sum(),
            sumx_sq: sumx * sumx,
        }
    }

    pub fn y(&self, x: f32) -> f32 {
        self.intercept + self.slope * x
    }

    pub fn update_from(&mut self, data: &[f32; NUM_SAMPLES]) {
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;

        for (i, x) in data.iter().enumerate() {
            sum_y += x;
            sum_xy += x * i as f32;
        }

        let n = NUM_SAMPLES as f32;
        self.intercept =
            (sum_y * self.sum_xsq - self.sumx * sum_xy) / (n * self.sum_xsq - self.sumx_sq);

        self.slope = (n * sum_xy - self.sumx * sum_y) / (n * self.sum_xsq - self.sumx_sq);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test0() {
        let mut lr = Linreg::<3>::new();
        lr.update_from(&[0.0, 0.0, 0.0]);

        assert_eq!(lr.intercept, 0.0);
        assert_eq!(lr.slope, 0.0);

        assert_eq!(lr.y(123.0), 0.0);
    }

    #[test]
    fn test_0_45() {
        let mut lr = Linreg::<3>::new();
        lr.update_from(&[0.0, 1.0, 2.0]);
        assert_eq!(lr.intercept, 0.0);
        assert_eq!(lr.slope, 1.0);
        assert_eq!(lr.y(10.0), 10.0);
    }

    #[test]
    fn test_1_45() {
        let mut lr = Linreg::<3>::new();
        lr.update_from(&[1.0, 2.0, 3.0]);

        assert_eq!(lr.y(0.0), 1.0);

        assert_eq!(lr.intercept, 1.0);
        assert_eq!(lr.slope, 1.0);

        assert_eq!(lr.y(9.0), 10.0);
    }
}
