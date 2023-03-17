use std::ops::RangeInclusive;

use itertools::Itertools;

use crate::{
    BitAndMerge, BitOrMerge, BitSubMerge, BitXOrTee, Integer, Merge, NotIter, SortedDisjoint,
    UnionIter,
};

/// The trait used to provide methods common to iterators with the [`SortedDisjoint`] trait.
/// Methods include [`to_string`], [`equal`], [`bitor`] (union), [`bitand`] (intersection), [`bitxor`], [`sub`], [`not`].
///
/// [`to_string`]: SortedDisjointIterator::to_string
/// [`equal`]: SortedDisjointIterator::equal
/// [`bitor`]: SortedDisjointIterator::bitor
/// [`bitand`]: SortedDisjointIterator::bitand
/// [`bitxor`]: SortedDisjointIterator::bitxor
/// [`sub`]: SortedDisjointIterator::sub
/// [`not`]: SortedDisjointIterator::not
// !!!cmk0000 could equal be don't with PartialEq? and thus ==?
// !!!cmk0000 link to all methods and operators.
// !!!cmk0000 should the readme include a table or example, etc.
pub trait SortedDisjointIterator<T: Integer>:
    Iterator<Item = RangeInclusive<T>> + SortedDisjoint + Sized
{
    // I think this is 'Sized' because will sometimes want to create a struct (e.g. BitOrIter) that contains a field of this type

    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator of their union.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::{CheckSortedDisjoint, RangeSetInt, SortedDisjointIterator};
    ///
    /// let a = CheckSortedDisjoint::new([1..=1].into_iter());
    /// let b = RangeSetInt::from([2..=2]).into_ranges();
    /// // CheckSortedDisjoint defines ops::bitor as SortedDisjointIterator::bitor so we can use '!'
    /// let union = a | b;
    /// assert_eq!(union.to_string(), "1..=2");
    /// ```
    #[inline]
    fn bitor<R>(self, other: R) -> BitOrMerge<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint,
    {
        UnionIter::new(Merge::new(self, other.into_iter()))
    }

    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator of their intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::{CheckSortedDisjoint, RangeSetInt, SortedDisjointIterator};
    ///
    /// let a = CheckSortedDisjoint::new([1..=2].into_iter());
    /// let b = RangeSetInt::from([2..=3]).into_ranges();
    /// // CheckSortedDisjoint defines ops::bitand as SortedDisjointIterator::bitand so we can use '&'
    /// let intersection = a & b;
    /// assert_eq!(intersection.to_string(), "2..=2");
    /// ```
    #[inline]
    fn bitand<R>(self, other: R) -> BitAndMerge<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint,
    {
        !(self.not().bitor(other.into_iter().not()))
    }

    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator of their set difference.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::{CheckSortedDisjoint, RangeSetInt, SortedDisjointIterator};
    ///
    /// let a = CheckSortedDisjoint::new([1..=2].into_iter());
    /// let b = RangeSetInt::from([2..=3]).into_ranges();
    /// // CheckSortedDisjoint defines ops::sub as SortedDisjointIterator::sub so we can use '-'
    /// let diff = a - b;
    /// assert_eq!(diff.to_string(), "1..=1");
    /// ```
    #[inline]
    fn sub<R>(self, other: R) -> BitSubMerge<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint,
    {
        (self.not().bitor(other.into_iter())).not()
    }

    /// Given a [`SortedDisjoint`] iterator, efficiently returns a [`SortedDisjoint`] iterator of its complement.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::{CheckSortedDisjoint, RangeSetInt, SortedDisjointIterator};
    ///
    /// let a = CheckSortedDisjoint::new([-10i16..=0, 1000..=2000].into_iter());
    /// // CheckSortedDisjoint defines ops::not as SortedDisjointIterator::not so we can use '!'
    /// let complement = !a;
    /// assert_eq!(complement.to_string(), "-32768..=-11, 1..=999, 2001..=32767");
    /// ```
    #[inline]
    fn not(self) -> NotIter<T, Self> {
        NotIter::new(self)
    }

    // !!! cmk test the speed of this
    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator of their set difference.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::{CheckSortedDisjoint, RangeSetInt, SortedDisjointIterator};
    ///
    /// let a = CheckSortedDisjoint::new([1..=2].into_iter());
    /// let b = RangeSetInt::from([2..=3]).into_ranges();
    /// // CheckSortedDisjoint defines ops::sub as SortedDisjointIterator::sub so we can use '^'
    /// let sym_diff = a ^ b;
    /// assert_eq!(sym_diff.to_string(), "1..=1, 3..=3");
    /// ```
    #[inline]
    fn bitxor<R>(self, other: R) -> BitXOrTee<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint,
    {
        let (lhs0, lhs1) = self.tee();
        let (rhs0, rhs1) = other.into_iter().tee();
        lhs0.sub(rhs0) | rhs1.sub(lhs1)
    }

    // cmk rule: Prefer IntoIterator to Iterator
    /// Given two [`SortedDisjoint`] iterators, efficiently tells if they are equal. Unlike most equality testing in Rust,
    /// this method takes ownership of the iterators and consumes them.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::{CheckSortedDisjoint, RangeSetInt, SortedDisjointIterator};
    ///
    /// let a = CheckSortedDisjoint::new([1..=2].into_iter());
    /// let b = RangeSetInt::from([1..=2]).into_ranges();
    /// assert!(a.equal(b));
    /// ```
    fn equal<R>(self, other: R) -> bool
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint,
    {
        itertools::equal(self, other)
    }

    // cmk rule: You can't define traits on combinations of traits, so use this method to define methods on traits
    /// Given a [`SortedDisjoint`] iterators, produces a string version. Unlike most `to_string` and `fmt` in Rust,
    /// this method takes ownership of the iterator and consumes it.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::{CheckSortedDisjoint, RangeSetInt, SortedDisjointIterator};
    ///
    /// let a = CheckSortedDisjoint::new([1..=2].into_iter());
    /// assert_eq!(a.to_string(), "1..=2");
    /// ```
    fn to_string(self) -> String {
        self.map(|range| {
            let (start, end) = range.into_inner();
            format!("{start}..={end}") // cmk could we format RangeInclusive directly?
        })
        .join(", ")
    }
}

// cmk0 explain why this is needed
impl<T, I> SortedDisjointIterator<T> for I
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
}
