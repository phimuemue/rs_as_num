//! This module implements a very opinionated approach to converting numbers.
//!
//! Imagine you have a function `return_u32` returning an `u32`, and you would like to pass its return value into some
//! other function `take_i8`, taking an `i8`:
//!
//! Then, the compiler (correctly) complains as soon as you write `take_i8(return_u32())`.
//! I came into those situations frequently, so I simply changed it to 
//! `take_i8(return_u32() as i8)`. However, when doing so, I implicitly assumed that the semantic
//! meaning of the number does not change, i.e. I assume that `i8` is capable of representing the
//! exact same value that `return_u32` gives me (which is not the case in the example shown).
//!
//! This module enables you to write the following:
//!
//! ```
//! use as_num::AsNum; // AsNum is the trait enabling the conversions
//! fn return_u32() -> u32 {
//!     42
//! }
//! fn take_i8(_i: i8) {
//! }
//! take_i8(return_u32().as_num())
//! ```
//!
//! `as_num` converts its argument into the destination type, thereby checking whether the
//! conversion can be done without loss of data.
//!
//! It tries to follow a similar approach to the one that is chosen with e.g. "normal addition" and `checked_add`:
//! It offers one method `as_num` that does the conversion (at last going down to Rust's `as`), and
//! `debug_assert`s that the conversion is lossless.
//! In addition to `as_num`, it offers a method `checked_as_num`, returning an `Option`.
//!
//! This module implements conversion for any combination of the following types:
//! `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64`, `usize`, `f32`, `f64`.
//!
//! The function `as_num` `debug_assert`s that the destination value is convertible back to the
//! exact same source value.
//!
//! That, in particular, means that converting floating-point to integral numbers can only be done
//! with `as_num` if the source is already been rounded to some integral number.

use std::mem;
use std::fmt::Debug;

// heavily inspired by http://rust-num.github.io/num/src/num_traits/cast.rs.html

// TODO rust i128/u128
type LargestSignedType = i64;
type LargestUnsignedType = u64;

pub trait SignedInt : Sized + Copy {
    #[inline(always)]
    fn min() -> LargestSignedType;
    #[inline(always)]
    fn max() -> LargestSignedType;
}

pub trait UnsignedInt : Sized + Copy {
    #[inline(always)]
    fn min() -> LargestUnsignedType;
    #[inline(always)]
    fn max() -> LargestUnsignedType;
}

macro_rules! impl_min_max {
    ($num_trait: ident, $largest_type_same_signedness: ty,) => {};
    ($num_trait: ident, $largest_type_same_signedness: ty, $t: ident, $($ts: ident,)*) => {
        impl $num_trait for $t {
            #[inline(always)]
            fn min() -> $largest_type_same_signedness {
                use std::$t;
                $t::MIN as $largest_type_same_signedness
            }
            #[inline(always)]
            fn max() -> $largest_type_same_signedness {
                use std::$t;
                $t::MAX as $largest_type_same_signedness
            }
        }
        impl_min_max!($num_trait, $largest_type_same_signedness, $($ts,)*);
    };
}

impl_min_max!(SignedInt, LargestSignedType, i8, i16, i32, i64, isize,);
impl_min_max!(UnsignedInt, LargestUnsignedType, u8, u16, u32, u64, usize,);

pub trait AsNumInternal<Dest> : Copy {
    #[inline(always)]
    fn is_safely_convertible(self) -> bool;
    #[inline(always)]
    fn as_num_internal(self) -> Dest;
}

pub trait AsNum {
    #[inline(always)]
    fn as_num<Dest>(self) -> Dest
        where Self: AsNumInternal<Dest>,
              Dest: AsNumInternal<Self>,
              Dest: Debug;
    #[inline(always)]
    fn checked_as_num<Dest>(self) -> Option<Dest>
        where Self: AsNumInternal<Dest>,
              Dest: AsNumInternal<Self>,
              Dest: Debug;
    #[inline(always)]
    fn assert_convertible_back<Dest>(self)
        where Self: AsNumInternal<Dest>,
              Dest: AsNumInternal<Self>,
              Dest: Debug;
}

macro_rules! impl_TAsNum {
    () => {};
    ($t: ident, $($ts: ident,)*) => {
        impl AsNum for $t {
            #[inline(always)]
            fn assert_convertible_back<Dest>(self)
                where Self: AsNumInternal<Dest>,
                      Dest: AsNumInternal<Self>,
                      Dest: Debug,
            {
                let dst : Dest = self.as_num_internal();
                let src : Self = dst.as_num_internal();
                debug_assert!(self==src, "{:?} {:?} was converted to {:?}, whose back-conversion yields {:?}", self, stringify!($t), dst, src);
            }
            #[inline(always)]
            fn as_num<Dest>(self) -> Dest
                where Self: AsNumInternal<Dest>,
                      Dest: AsNumInternal<Self>,
                      Dest: Debug,
            {
                debug_assert!(self.is_safely_convertible(), "{} not safely convertible", self);
                self.assert_convertible_back::<Dest>();
                self.as_num_internal()
            }
            #[inline(always)]
            fn checked_as_num<Dest>(self) -> Option<Dest>
                where Self: AsNumInternal<Dest>,
                      Dest: AsNumInternal<Self>,
                      Dest: Debug,
            {
                if self.is_safely_convertible() {
                    self.assert_convertible_back::<Dest>();
                    Some(self.as_num_internal())
                } else {
                    None
                }
            }
        }
        impl_TAsNum!($($ts,)*);
    };
}
impl_TAsNum!(
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
    f32, f64,
);

macro_rules! impl_signed_to_signed_internal {
    ($src: ident, $dest: ident) => {
        impl AsNumInternal<$dest> for $src {
            #[inline(always)]
            fn is_safely_convertible(self) -> bool {
                mem::size_of::<$src>() <= mem::size_of::<$dest>()
                || {
                    debug_assert!(mem::size_of::<Self>() <= mem::size_of::<LargestSignedType>());
                    let n = self as LargestSignedType;
                    <$dest as SignedInt>::min() <= n && n <= <$dest as SignedInt>::max()
                }
            }
            #[inline(always)]
            fn as_num_internal(self) -> $dest {
                self as $dest
            }
        }
    };
}

macro_rules! impl_signed_to_signed {
    ($src: ident,) => {};
    ($src: ident, $dest: ident, $($dests: ident,)*) => {
        impl_signed_to_signed_internal!($src, $dest);
        impl_signed_to_signed_internal!($dest, $src);
        impl_signed_to_signed!($src, $($dests,)*);
    };
}

macro_rules! impl_signed_to_unsigned_internal {
    ($src: ident, $dest: ident) => {
        impl AsNumInternal<$dest> for $src {
            #[inline(always)]
            fn is_safely_convertible(self) -> bool {
                0<=self && self as LargestUnsignedType <= <$dest as UnsignedInt>::max()
            }
            #[inline(always)]
            fn as_num_internal(self) -> $dest {
                self as $dest
            }
        }
    }
}

macro_rules! impl_signed_to_unsigned {
    ($src: ident,) => {};
    ($src: ident, $dest: ident, $($dests: ident,)*) => {
        impl_signed_to_unsigned_internal!($src, $dest);
        impl_unsigned_to_signed_internal!($dest, $src);
        impl_signed_to_unsigned!($src, $($dests,)*);
    }
}

macro_rules! impl_unsigned_to_signed_internal {
    ($src: ident, $dest: ident) => {
        impl AsNumInternal<$dest> for $src {
            #[inline(always)]
            fn is_safely_convertible(self) -> bool {
                self as LargestUnsignedType <= <$dest as SignedInt>::max() as LargestUnsignedType
            }
            #[inline(always)]
            fn as_num_internal(self) -> $dest {
                self as $dest
            }
        }
    };
}

macro_rules! impl_unsigned_to_signed {
    ($src: ident,) => {};
    ($src: ident, $dest: ident, $($dests: ident,)*) => {
        impl_unsigned_to_signed_internal!($src, $dest);
        impl_signed_to_unsigned_internal!($dest, $src);
        impl_unsigned_to_signed!($src, $($dests,)*);
    };
}

macro_rules! impl_unsigned_to_unsigned_internal {
    ($src: ident, $dest: ident) => {
        impl AsNumInternal<$dest> for $src {
            #[inline(always)]
            fn is_safely_convertible(self) -> bool {
                mem::size_of::<$src>() <= mem::size_of::<$dest>()
                    || self as LargestUnsignedType <= <$dest as UnsignedInt>::max()
            }
            #[inline(always)]
            fn as_num_internal(self) -> $dest {
                self as $dest
            }
        }
    };
}

macro_rules! impl_unsigned_to_unsigned {
    ($src: ident,) => {};
    ($src: ident, $dest: ident, $($dests: ident,)*) => {
        impl_unsigned_to_unsigned_internal!($src, $dest);
        impl_unsigned_to_unsigned_internal!($dest, $src);
        impl_unsigned_to_unsigned!($src, $($dests,)*);
    };
}

macro_rules! impl_integral_conversions {
    ((), ($($unsigneds: ident,)*)) => {};
    (($signed: ident, $($signeds: ident,)*), ($unsigned: ident, $($unsigneds: ident,)*)) => {
        impl_signed_to_signed_internal!($signed, $signed);
        impl_signed_to_signed!($signed, $($signeds,)*);
        impl_signed_to_unsigned_internal!($signed, $unsigned);
        impl_signed_to_unsigned!($signed, $($unsigneds,)*);
        impl_unsigned_to_signed_internal!($unsigned, $signed);
        impl_unsigned_to_signed!($unsigned, $($signeds,)*);
        impl_unsigned_to_unsigned_internal!($unsigned, $unsigned);
        impl_unsigned_to_unsigned!($unsigned, $($unsigneds,)*);
        impl_integral_conversions!(($($signeds,)*), ($($unsigneds,)*));
    };
}

impl_integral_conversions!(
    (i8, i16, i32, i64, isize,),
    (u8, u16, u32, u64, usize,)
);

macro_rules! impl_integral_to_float_internal {
    ($flt: ident,) => {};
    ($flt: ident, $int: ident, $($ints: ident,)*) => {
        impl AsNumInternal<$flt> for $int {
            #[inline(always)]
            fn is_safely_convertible(self) -> bool {
                true // assume convertability until we encounter counter example in practice
            }
            #[inline(always)]
            fn as_num_internal(self) -> $flt {
                self as $flt
            }
        }
        impl AsNumInternal<$int> for $flt {
            #[inline(always)]
            fn is_safely_convertible(self) -> bool {
                let dst : $int = self.as_num_internal();
                let src : Self = dst.as_num_internal();
                self==src
            }
            #[inline(always)]
            fn as_num_internal(self) -> $int {
                self as $int
            }
        }
        impl_integral_to_float_internal!($flt, $($ints,)*);
    };
}
macro_rules! impl_integral_to_float {
    ($flt: ident) => {
        impl_integral_to_float_internal!($flt,
            i8, i16, i32, i64, isize,
            u8, u16, u32, u64, usize,
        );
    };
}
impl_integral_to_float!(f32);
impl_integral_to_float!(f64);

type LargestFloatType = f64;
macro_rules! impl_float_to_float_internal {
    ($src: ident, $dest: ident) => {
        impl AsNumInternal<$dest> for $src {
            #[inline(always)]
            fn is_safely_convertible(self) -> bool {
                mem::size_of::<$src>() <= mem::size_of::<$dest>() 
                || {
                    // Make sure the value is in range for the cast.
                    // NaN and +-inf are cast as they are.
                    let f = self as LargestFloatType;
                    !f.is_finite() || {
                        let max_value: $dest = ::std::$dest::MAX;
                        -max_value as LargestFloatType <= f && f <= max_value as LargestFloatType
                    }
                }
            }
            #[inline(always)]
            fn as_num_internal(self) -> $dest {
                self as $dest
            }
        }
    }
}
macro_rules! impl_float_to_float {
    ($src: ident,) => {};
    ($src: ident, $dest: ident, $($dests: ident,)*) => {
        impl_float_to_float_internal!($src, $dest);
        impl_float_to_float_internal!($dest, $src);
        impl_float_to_float!($src, $($dests,)*);
    };
}
impl_float_to_float!(f32, f64,);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_as_num() {
        // we assume that isize/usize occupy at least 32 bit (i.e. 4 byte)
        // TODO tests: improve
        assert_eq!(4i8, 4i8.checked_as_num().unwrap());
        assert_eq!(4usize, 4u16.as_num());
        assert_eq!(4i32, 4usize.as_num());
        assert_eq!(256isize.checked_as_num::<u8>(), None);
        assert_eq!(4.3.checked_as_num::<isize>(), None);
    }

    #[test]
    fn test_ulargest_to_ilargest() {
        assert_eq!(
            ((<LargestSignedType as SignedInt>::max() as LargestUnsignedType)+1).checked_as_num::<LargestSignedType>(), None
        )
    }
}
