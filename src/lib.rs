#![deny(missing_docs)]
#![cfg_attr(feature = "oibit", feature(optin_builtin_traits))]

//! A safe approach to using `#[repr(packed)]` data.
//!
//! See `nue_macros` for the automagic `#[packed]` attribute.

use std::mem::{transmute, uninitialized, forget, align_of, size_of};
use std::ptr::write;
use std::marker::PhantomData;

/// A marker trait indicating that a type has an alignment of `1`.
///
/// In general, only applies to `()`, `bool`, `i8`, `u8`, and any types that
/// contain only members of these types.
pub unsafe trait Unaligned { }

/// A type alias that represents the unaligned type of `T`.
pub type Un<T> = <T as Aligned>::Unaligned;

/// Determines whether a pointer or reference is correctly aligned for type `T`.
pub fn is_aligned_for<T, U>(ptr: *const U) -> bool {
    ptr as usize % align_of::<T>() == 0
}

/// Determines whether a slice is correctly aligned for type `T`.
pub fn is_aligned_for_slice<T, U>(slice: &[U]) -> bool {
    is_aligned_for::<T, _>(slice.as_ptr())
}

/// Calculates the total byte size of a slice.
pub fn size_of_slice<T>(slice: &[T]) -> usize {
    slice.len() * size_of::<T>()
}

/// A trait for converting types with alignments greater than `1`
/// into their unaligned equivalent.
pub unsafe trait Aligned: Sized {
    /// An unaligned representation of this type. Usually a u8 array of the
    /// same size.
    type Unaligned: Unaligned + Sized + Copy;

    /// Determines whether an unaligned representation of this type is aligned.
    #[inline]
    fn is_aligned(unaligned: &Self::Unaligned) -> bool {
        is_aligned_for::<Self, _>(unaligned)
    }

    /// Borrows the value as unaligned.
    #[inline]
    fn as_unaligned(&self) -> &Self::Unaligned {
        unsafe { transmute(self) }
    }

    /// Mutably borrows the value as unaligned.
    #[inline]
    unsafe fn as_unaligned_mut(&mut self) -> &mut Self::Unaligned {
        transmute(self)
    }

    /// Borrows an unaligned type as an aligned value.
    ///
    /// Returns `None` if `unaligned` is not aligned.
    #[inline]
    fn from_unaligned_ref(unaligned: &Self::Unaligned) -> Option<&Self> {
        if Self::is_aligned(unaligned) {
            Some(unsafe { Self::from_unaligned_unchecked(unaligned) })
        } else {
            None
        }
    }

    /// Mutably borrows an unaligned type as an aligned value.
    ///
    /// Returns `None` if `unaligned` is not aligned.
    #[inline]
    unsafe fn from_unaligned_mut(unaligned: &mut Self::Unaligned) -> Option<&mut Self> {
        if Self::is_aligned(unaligned) {
            Some(Self::from_unaligned_mut_unchecked(unaligned))
        } else {
            None
        }
    }

    /// Borrows an unaligned type as an aligned value, without first checking the alignment.
    ///
    /// Causes undefined behaviour if used improprly!
    #[inline]
    unsafe fn from_unaligned_unchecked(unaligned: &Self::Unaligned) -> &Self {
        transmute(unaligned)
    }

    /// Mutably borrows an unaligned type as an aligned value, without first checking the alignment.
    ///
    /// Causes undefined behaviour if used improprly!
    #[inline]
    unsafe fn from_unaligned_mut_unchecked(unaligned: &mut Self::Unaligned) -> &mut Self {
        transmute(unaligned)
    }

    /// Converts a value to its unaligned representation without dropping `self`.
    #[inline]
    fn into_unaligned(self) -> Self::Unaligned {
        unsafe {
            let mut un: Self::Unaligned = uninitialized();
            write(&mut un, *self.as_unaligned());
            forget(self);
            un
        }
    }

    /// Creates a value from its unaligned representation.
    #[inline]
    unsafe fn from_unaligned(u: Self::Unaligned) -> Self {
        let mut s: Self = uninitialized();
        write(s.as_unaligned_mut(), u);
        s
    }

    #[doc(hidden)]
    unsafe fn __assert_unaligned() { }
}

/// A marker trait indicating that a type is `#[repr(packed)]`.
///
/// This means that all its members are packed or have an alignment of `1`,
/// and its memory layout is guaranteed to be in member declaration order.
pub unsafe trait Packed: Unaligned {
    #[doc(hidden)]
    fn __assert_unaligned() { }
}

#[cfg(feature = "oibit")]
mod impls {
    use super::Unaligned;

    unsafe impl Unaligned for .. { }

    // All primitives except the packed ones listed below
    impl !Unaligned for char { }
    impl !Unaligned for f32 { }
    impl !Unaligned for f64 { }
    impl !Unaligned for i16 { }
    impl !Unaligned for u16 { }
    impl !Unaligned for i32 { }
    impl !Unaligned for u32 { }
    impl !Unaligned for i64 { }
    impl !Unaligned for u64 { }
    impl !Unaligned for isize { }
    impl !Unaligned for usize { }
    impl<T> !Unaligned for *const T { }
    impl<T> !Unaligned for *mut T { }
    impl<'a, T> !Unaligned for &'a T { }
    impl<'a, T> !Unaligned for &'a mut T { }
}

#[cfg(not(feature = "oibit"))]
mod impls {
    use super::Unaligned;

    unsafe impl Unaligned for () { }
    unsafe impl Unaligned for i8 { }
    unsafe impl Unaligned for u8 { }
    unsafe impl Unaligned for bool { }
}

unsafe impl<T> Unaligned for PhantomData<T> { }

macro_rules! aligned_assert {
    ($t:ident) => {
        unsafe fn __assert_unaligned() {
            ::std::mem::forget(::std::mem::transmute::<$t, $t::Unaligned>(::std::mem::uninitialized()));
        }
    };
}

macro_rules! aligned_self {
    ($t:ty) => {
        unsafe impl Aligned for $t {
            type Unaligned = $t;
        }
    };
    ($($t:ty),*) => {
        $(
            aligned_self!($t);
        )*
    };
}

macro_rules! aligned_impl {
    ($t:ident: $s:expr) => {
        unsafe impl Aligned for $t {
            type Unaligned = [u8; $s];

            aligned_assert!(Self);
        }
    };
    ($($t:ident: $e:expr),*) => {
        $(
            aligned_impl!($t: $e);
        )*
    };
}

aligned_impl! {
    char: 4,
    f32: 4,
    f64: 8,
    i16: 2,
    u16: 2,
    i32: 4,
    u32: 4,
    i64: 8,
    u64: 8
}

aligned_self! {
    u8,
    i8,
    (),
    bool
}

#[cfg(target_pointer_width = "32")]
mod impl32 {
    use super::Aligned;
    aligned_impl! { isize: 4, usize: 4 }
    unsafe impl<T: Sized> Aligned for *const T { type Unaligned = [u8; 4]; aligned_assert!(Self); }
    unsafe impl<T: Sized> Aligned for *mut T { type Unaligned = [u8; 4]; aligned_assert!(Self); }
    unsafe impl<'a, T: Sized> Aligned for &'a T { type Unaligned = [u8; 4]; }
    unsafe impl<'a, T: Sized> Aligned for &'a mut T { type Unaligned = [u8; 4]; }
}

#[cfg(target_pointer_width = "64")]
mod impl64 {
    use super::Aligned;
    aligned_impl! { isize: 8, usize: 8 }
    unsafe impl<T: Sized> Aligned for *const T { type Unaligned = [u8; 8]; aligned_assert!(Self); }
    unsafe impl<T: Sized> Aligned for *mut T { type Unaligned = [u8; 8]; aligned_assert!(Self); }
    unsafe impl<'a, T: Sized> Aligned for &'a T { type Unaligned = [u8; 8]; }
    unsafe impl<'a, T: Sized> Aligned for &'a mut T { type Unaligned = [u8; 8]; }
}

// TODO: why does this conflict?
//unsafe impl<T: Unaligned + Sized + Copy> Aligned for T { type Unaligned = T; }

unsafe impl Packed for () { }
unsafe impl Packed for i8 { }
unsafe impl Packed for u8 { }
unsafe impl Packed for bool { }
unsafe impl<T> Packed for PhantomData<T> { }

macro_rules! packed_def {
    (=> $id_0:ident) => {};
    (=> $id_0:ident $(, $ids:ident)*) => {
        packed_def! {
            => $($ids),*
        }

        unsafe impl<$($ids: Unaligned),*> Unaligned for ($($ids),*,) { }

        // TODO: The language probably doesn't guarantee ordering for tuples, even when all are 1-aligned, so this is incorrect?
        unsafe impl<$($ids: Unaligned),*> Packed for ($($ids),*,) { }
    };
    ($($x:expr),*) => {
        $(
            unsafe impl<T: Unaligned> Unaligned for [T; $x] { }

            unsafe impl<T: Unaligned> Packed for [T; $x] { }
        )*
    };
}

unsafe impl<T: Aligned> Aligned for (T,) {
    type Unaligned = T::Unaligned;
}

unsafe impl<T: Aligned> Aligned for [T; 1] {
    type Unaligned = T::Unaligned;
}

unsafe impl<T: Aligned> Aligned for [T; 0] {
    type Unaligned = ();
}

packed_def! {
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
    0x40,
    0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x700, 0x800, 0x900, 0xa00, 0xb00, 0xc00, 0xd00, 0xe00, 0xf00,
    0x1000
}
packed_def! { => __, A, B, C, D, E, F, G, H, I, J, K }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn assert_packed() {
        fn is<T: Packed>() { }
        fn is_unaligned<T: Unaligned>() { }

        is::<()>();
        is::<u8>();
        is::<i8>();
        is::<bool>();
        is_unaligned::<(bool, u8)>();
    }

    #[test]
    fn unaligned_conversion() {
        let f = 0.5f32;
        let x = Aligned::into_unaligned(f);
        assert!(<f32 as Aligned>::is_aligned(&x));
        assert_eq!(&f, <f32 as Aligned>::from_unaligned_ref(&x).unwrap());
    }

    #[test]
    fn pointer_alignment() {
        use std::mem::align_of;

        let f = 0.5f32;
        assert!(is_aligned_for::<f32, _>(&f));
        assert!(is_aligned_for::<u8, _>(&f));
        if align_of::<f32>() > 1 {
            assert!(!is_aligned_for::<f32, _>((&f as *const _ as usize + 1) as *const u8));
        }
    }
}
