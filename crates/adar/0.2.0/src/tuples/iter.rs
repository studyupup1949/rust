pub struct TupleIterator<'a, T, const N: usize> {
    items: [&'a T; N],
    idx: usize,
}

impl<'a, T, const N: usize> Iterator for TupleIterator<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < N {
            let out = self.items[self.idx];
            self.idx += 1;
            Some(out)
        } else {
            None
        }
    }
}

pub trait TupleIter<T, const N: usize> {
    fn iter(&self) -> TupleIterator<'_, T, N>;
}

macro_rules! impl_tuple_iter {
    ($len:expr, ( $( $T:ident => $idx:tt ),* )) => {
        impl<T> TupleIter<T, $len> for ( $( $T ,)* ) {
            fn iter(&self) -> TupleIterator<'_, T, $len> {
                TupleIterator {
                    items: [ $( &self.$idx ),* ],
                    idx: 0,
                }
            }
        }
    };
}

impl_tuple_iter!(1, (T=>0));
impl_tuple_iter!(2, (T=>0, T=>1));
impl_tuple_iter!(3, (T=>0, T=>1, T=>2));
impl_tuple_iter!(4, (T=>0, T=>1, T=>2, T=>3));
impl_tuple_iter!(5, (T=>0, T=>1, T=>2, T=>3, T=>4));
impl_tuple_iter!(6, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5));
impl_tuple_iter!(7, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5, T=>6));
impl_tuple_iter!(8, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5, T=>6, T=>7));
impl_tuple_iter!(9, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5, T=>6, T=>7, T=>8));
impl_tuple_iter!(10, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5, T=>6, T=>7, T=>8, T=>9));
impl_tuple_iter!(11, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5, T=>6, T=>7, T=>8, T=>9, T=>10));
impl_tuple_iter!(12, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5, T=>6, T=>7, T=>8, T=>9, T=>10, T=>11));
impl_tuple_iter!(13, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5, T=>6, T=>7, T=>8, T=>9, T=>10, T=>11, T=>12));
impl_tuple_iter!(14, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5, T=>6, T=>7, T=>8, T=>9, T=>10, T=>11, T=>12, T=>13));
impl_tuple_iter!(15, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5, T=>6, T=>7, T=>8, T=>9, T=>10, T=>11, T=>12, T=>13, T=>14));
impl_tuple_iter!(16, (T=>0, T=>1, T=>2, T=>3, T=>4, T=>5, T=>6, T=>7, T=>8, T=>9, T=>10, T=>11, T=>12, T=>13, T=>14, T=>15));

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tuple_iter3() {
        let tuple = (0, 1, 2);

        let mut iter = tuple.iter();
        assert_eq!(iter.next(), Some(&tuple.0));
        assert_eq!(iter.next(), Some(&tuple.1));
        assert_eq!(iter.next(), Some(&tuple.2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_tuple_iter16() {
        let tuple = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

        let mut iter = tuple.iter();
        assert_eq!(iter.next(), Some(&tuple.0));
        assert_eq!(iter.next(), Some(&tuple.1));
        assert_eq!(iter.next(), Some(&tuple.2));
        assert_eq!(iter.next(), Some(&tuple.3));
        assert_eq!(iter.next(), Some(&tuple.4));
        assert_eq!(iter.next(), Some(&tuple.5));
        assert_eq!(iter.next(), Some(&tuple.6));
        assert_eq!(iter.next(), Some(&tuple.7));
        assert_eq!(iter.next(), Some(&tuple.8));
        assert_eq!(iter.next(), Some(&tuple.9));
        assert_eq!(iter.next(), Some(&tuple.10));
        assert_eq!(iter.next(), Some(&tuple.11));
        assert_eq!(iter.next(), Some(&tuple.12));
        assert_eq!(iter.next(), Some(&tuple.13));
        assert_eq!(iter.next(), Some(&tuple.14));
        assert_eq!(iter.next(), Some(&tuple.15));
        assert_eq!(iter.next(), None);
    }
}
