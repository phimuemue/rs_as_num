This module implements a very opinionated approach to converting numbers.

Imagine you have a function `return_u32`, and you would like to pass its return value into some
other function take_i8:

```
fn return_u32() -> u32 {
    257
}
fn take_i8(i: i8) {
}
```

Then, the compiler (correctly) complains as soon as you write `take_i8(return_u32())`.
I came into those situations frequently, so I simply changed it to 
`take_i8(return_u32() as i8)`. However, when doing so, I implicitly assumed that the semantic
meaning of the number does not change, i.e. I assume that `i8` is capable of representing the
exact same value that `return_u32` gives me (which is not the case in the example shown).

This module enables you to write the following:

```
use as_num::TAsNum; // TAsNum is the trait enabling the conversions
take_i8(return_u32().as_num())
```

`as_num` converts its argument into the destination type, thereby checking whether the
conversion can be done without loss of data.

It tries to follow a similar approach to the one that is chosen with e.g. "normal addition" and `checked_add`:
It offers one method `as_num` that does the conversion (at last going down to Rust's `as`), and
`debug_assert`s that the conversion is lossless.
In addition to `as_num`, it offers a method `checked_as_num`, returning an `Option`.

This module implements conversion for any combination of the following types:
`i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64`, `usize`, `f32`, `f64`.

The function `as_num` `debug_assert`s that the destination value is convertible back to the
exact same source value.

That, in particular, means that converting floating-point to integral numbers can only be done
with `as_num` if the source is already been rounded to some integral number.

