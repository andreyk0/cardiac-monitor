//! Tiny curcular buffer

#[derive(Copy, Clone, Debug)]
pub struct Circ<T, const COUNT: usize> {
    pub data: [T; COUNT],
    next: usize,
}

impl<T, const COUNT: usize> Circ<T, COUNT>
where
    T: Copy,
{
    pub fn new(zero: T) -> Self {
        Circ {
            data: [zero; COUNT],
            next: 0,
        }
    }

    pub fn add(&mut self, s: T) {
        self.data[self.next] = s;
        self.next = wrap_next::<COUNT>(self.next);
    }

    pub fn iter<'a>(&'a self) -> CircIter<'a, T, COUNT> {
        CircIter {
            circ: self,
            idx: self.next,
            done: false,
        }
    }
}

impl<'a, T, const COUNT: usize> IntoIterator for &'a Circ<T, COUNT>
where
    T: Copy,
{
    type Item = T;

    type IntoIter = CircIter<'a, T, COUNT>;

    fn into_iter(self) -> Self::IntoIter {
        CircIter {
            circ: self,
            idx: self.next,
            done: false,
        }
    }
}

pub struct CircIter<'a, T, const COUNT: usize> {
    circ: &'a Circ<T, COUNT>,
    idx: usize,
    done: bool,
}

impl<'a, T, const COUNT: usize> Iterator for CircIter<'a, T, COUNT>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let res = self.circ.data[self.idx];
            self.idx = wrap_next::<COUNT>(self.idx);
            if self.idx == self.circ.next {
                self.done = true;
            }
            Some(res)
        }
    }
}

#[inline]
fn wrap_next<const COUNT: usize>(n: usize) -> usize {
    let n1 = n + 1;
    if n1 >= COUNT {
        0
    } else {
        n1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut c = Circ::<u32, 3>::new(0);
        c.add(1);
        c.add(2);
        c.add(3);

        assert_eq!(c.data, [1, 2, 3]);

        c.add(4);
        c.add(5);
        assert_eq!(c.data, [4, 5, 3]);
    }

    #[test]
    fn test_iter() {
        let mut c = Circ::<u32, 3>::new(0);

        let all: Vec<u32> = c.iter().collect();
        assert_eq!(vec![0, 0, 0], all);

        c.add(1);
        let all: Vec<u32> = c.iter().collect();
        assert_eq!(vec![0, 0, 1], all);

        c.add(2);
        let all: Vec<u32> = c.iter().collect();
        assert_eq!(vec![0, 1, 2], all);

        c.add(3);
        c.add(4);

        let all: Vec<u32> = c.iter().collect();
        assert_eq!(vec![2, 3, 4], all);
    }
}
