use morton;
use std::iter::{Iterator, FusedIterator,ExactSizeIterator};

fn lol(buffer: &mut [f32], width: usize, height: usize) {
}
/*
struct Morton <'a, T:'a> {
    v: & 'a mut [T],
    current: usize,
    width: usize,
    height: usize,
}

impl <'a, T> Morton<'a, T> {
    fn morton(v: & 'a mut [T], width: usize, height: usize) -> Morton<'a, T> {
        // TODO make this a maybe?
        assert!(v.len() == width * height && v.len() % 2 == 0);
        Morton{ v, current: 0, width, height }
    }
}

impl <'a, T> Iterator for Morton<'a , T> {
    type Item = & 'a mut T;

    #[inline]
    fn next(&mut self) -> Option<& 'a mut T> {
        if self.current >= self.width * self.height {
            None
        } else {
            let (x,y) = morton::deinterleave_morton(self.current as u32);
            let x = x as usize;
            let y = y as usize;

            let k : & 'a mut T = unsafe {self.v.get_unchecked_mut(x + y * self.height) };
            self.current += 1;
            Some(k)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.v.is_empty() {
            (0, Some(0))
        } else {
            let n = self.v.len();
            (n, Some(n))
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        None
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        None 
    }
}

impl <'a, T> FusedIterator for Morton<'a,  T>{}

impl <'a, T> ExactSizeIterator for Morton<'a,  T> {
    fn len(&self) -> usize {
        self.width * self.height
    }
    fn is_empty(&self) -> bool {
        self.current as usize == self.width * self.height
    }
}*/
