//! Ad-hoc exact size owning iterator macro and other optional utils
//!
//! The macro can be used exactly as `vec!`.

#[cfg(feature="maybe-many")] pub mod maybe;

/// A bespoke iterator type with an exact size over elements.
///
/// This has an advantage over using simple inline slices (e.g `[1,2,3]`) as they currently cannot implement `IntoIterator` properly, and will always yield references.
/// This can be a hinderance when you want an ad-hoc iterator of non-`Copy` items.
///
/// The iterator types created from this macro consume the values.
/// It has the advantage over `vec![].into_iter()` as there are no heap allocations needed and size of the iterator is known at compile time.
///
/// # Example
/// ```
/// # use ad_hoc_iter::iter;
/// let sum: i32 = iter![1, 2].chain(3..=4).chain(iter![5, 6]).sum();
/// assert_eq!(sum, 21);
/// ```
/// # Functions
/// The iterators returned from this method have these associated functions:
///
/// ## The length of the whole iterator
/// ```ignore
/// pub const fn len(&self) -> usize
/// ```
///
/// ## The rest of the iterator that has not been consumed.
/// ```ignore
/// pub fn rest(&self) -> &[T]
/// ```
///
/// ##  The whole array.
///  All values that have not been consumed are initialised, values that have been consumed are uninitialised.
/// ```ignore
/// pub fn array(&self) -> &[MaybeUninit<T>; Self::LEN]
/// ```
///
/// ## How many items have since been consumed.
/// ```ignore
/// pub const fn consumed(&self) -> usize
/// ```
#[macro_export] macro_rules! iter {
    (@) => (0usize);
    (@ $x:tt $($xs:tt)* ) => (1usize + $crate::iter!(@ $($xs)*));

    () => {
	{
	    #[derive(Debug)]
	    struct Empty;
	    impl Iterator for Empty
	    {
		type Item = std::convert::Infallible;
		fn next(&mut self) -> Option<Self::Item>
		{
		    None
		}

		fn size_hint(&self) -> (usize, Option<usize>)
		{
		    (0, Some(0))
		}
	    }
	    impl ::std::iter::FusedIterator for Empty{}
	    impl ::std::iter::ExactSizeIterator for Empty{}

	    Empty
	}
    };
    
    ($($value:expr),* $(,)?) => {
	{
	    use ::std::mem::MaybeUninit;
	    use ::std::ops::Drop;
	    struct Arr<T>([MaybeUninit<T>; $crate::iter!(@ $($value)*)], usize);
	    impl<T> Arr<T>
	    {
		#![allow(dead_code)]
		/// The length of the whole iterator
		const LEN: usize = $crate::iter!(@ $($value)*);

		/// The length of the whole iterator
		// This exists as an associated function because this type is opaque.
		#[inline] pub const fn len(&self) -> usize
		{
		    Self::LEN
		}
		
		/// Consume this iterator into the backing buffer.
		///
		/// # Safety
		/// Non-consumed items are safe to `assume_init`. However, items that have already been consumed are uninitialised.
		fn into_inner(self) -> [MaybeUninit<T>; $crate::iter!(@ $($value)*)]
		{
		    //XXX: We will have to do something really unsafe for this to work on stable...
		    todo!()
		}

		/// The rest of the iterator that has not been consumed.
		///
		// # Safety
		// All values in this slice are initialised.
		#[inline] pub fn rest(&self) -> &[T]
		{
		    let slice = &self.0[self.1..];
		    //std::mem::MaybeUninit::slice_get_ref(&self.0[self.1..]) //nightly only...
		    
		    unsafe { &*(slice as *const [std::mem::MaybeUninit<T>] as *const [T]) }
		}

		/// The whole array.
		///
		/// # Safety
		/// All values that have not been consumed are initialised, values that have been consumed are uninitialised.
		#[inline] pub fn array(&self) -> &[MaybeUninit<T>; $crate::iter!(@ $($value)*)]
		{
		    &self.0
		}

		/// How many items have since been consumed.
		pub const fn consumed(&self) -> usize
		{
		    self.1
		}
	    }
	    impl<T> Iterator for Arr<T>
	    {
		type Item = T;
		fn next(&mut self) -> Option<Self::Item>
		{
		    if self.1 >= self.0.len() {
			None
		    } else {
			//take one
			let one = unsafe {
			    ::std::mem::replace(&mut self.0[self.1], MaybeUninit::uninit()).assume_init()
			};
			self.1+=1;
			Some(one)
		    }
		}

		#[inline] fn size_hint(&self) -> (usize, Option<usize>)
		{
		    (Self::LEN, Some(Self::LEN))
		}
	    }
	    impl<T> ::std::iter::FusedIterator for Arr<T>{}
	    impl<T> ::std::iter::ExactSizeIterator for Arr<T>{}
	    
	    impl<T> Drop for Arr<T>
	    {
		fn drop(&mut self) {
		    if ::std::mem::needs_drop::<T>() {
			for idx in self.1..self.0.len() {
			    unsafe {
				::std::mem::replace(&mut self.0[idx], MaybeUninit::uninit()).assume_init();
			    }
			}
		    }
		}
	    }

	    Arr([$(MaybeUninit::new($value)),*], 0)
	}
    }
}

#[cfg(test)]
mod tests
{
    #[test]
    fn iter_over()
    {
	const EXPECT: usize = 10 + 9 + 8 + 7 + 6 + 5 + 4 + 3 + 2 + 1;
	let iter = iter![10,9,8,7,6,5,4,3,2,1];
	
	assert_eq!(iter.len(), 10);
	assert_eq!(iter.sum::<usize>(), EXPECT);
	
    }

    
    macro_rules! string {
	($val:literal) => (String::from($val));
    }
    #[test]
    fn non_copy()
    {

	const EXPECT: &str = "Hello world!";
	let whole: String = iter![string!("Hell"), string!("o "), string!("world"), string!("!")].collect();
	assert_eq!(EXPECT, &whole[..]);
    }

    #[test]
    fn empty()
    {
	let empty: Vec<_> = iter![].collect();
	assert_eq!(empty.len(), 0);
    }

    #[test]
    fn assoc()
    {
	let mut iter = iter![1,2,3,4];

	assert_eq!(iter.len(), 4);
	iter.next();
	assert_eq!(iter.consumed(), 1);
	assert_eq!(iter.rest(), &[2,3,4]);

	
	let mut iter = iter![string!("Hell"), string!("o "), string!("world"), string!("!")];
	
	iter.next();
	iter.next();
	
	assert_eq!(iter.rest().iter().map(|x| x.as_str()).collect::<Vec<_>>().as_slice(), &["world", "!"]);
    }
}
