// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(non_snake_case)]

//! This crate provides wrappers around the underlying CoreFoundation
//! types and functions that are available on Apple's operating systems.
//!
//! It also provides a framework for other crates to use when wrapping
//! other frameworks that use the CoreFoundation framework.

use crate::base::TCFType;

pub unsafe trait ConcreteCFType: TCFType {}

/// Declare a Rust type that wraps an underlying CoreFoundation type.
///
/// This will provide an implementation of `Drop` using [`CFRelease`].
/// The type must have an implementation of the [`TCFType`] trait, usually
/// provided using the [`impl_TCFType`] macro.
///
/// ```
/// use core_foundation::{declare_TCFType, impl_TCFType};
/// // Make sure that the `TCFType` trait is in scope.
/// use core_foundation::base::{CFTypeID, TCFType};
///
/// extern "C" {
///     // We need a function that returns the `CFTypeID`.
///     pub fn ShrubberyGetTypeID() -> CFTypeID;
/// }
///
/// pub struct __Shrubbery {}
/// // The ref type must be a pointer to the underlying struct.
/// pub type ShrubberyRef = *const __Shrubbery;
///
/// declare_TCFType!(Shrubbery, ShrubberyRef);
/// impl_TCFType!(Shrubbery, ShrubberyRef, ShrubberyGetTypeID);
/// # fn main() {}
/// ```
///
/// [`CFRelease`]: https://developer.apple.com/documentation/corefoundation/1521153-cfrelease
/// [`TCFType`]: base/trait.TCFType.html
/// [`impl_TCFType`]: macro.impl_TCFType.html
#[macro_export]
macro_rules! declare_TCFType {
    (
        $(#[$doc:meta])*
        $ty:ident, $raw:ident
    ) => {
        declare_TCFType!($(#[$doc])* $ty<>, $raw);
    };

    (
        $(#[$doc:meta])*
        $ty:ident<$($p:ident $(: $bound:path)*),*>, $raw:ident
    ) => {
        $(#[$doc])*
        pub struct $ty<$($p $(: $bound)*),*>($raw, $(::std::marker::PhantomData<$p>),*);

        #[allow(unused_imports)]
        impl<$($p $(: $bound)*),*> Drop for $ty<$($p),*> {
            fn drop(&mut self) {
                use $crate::base::TCFType;
                unsafe { $crate::base::CFRelease(self.as_CFTypeRef()) }
            }
        }
    };
}

/// Provide an implementation of the [`TCFType`] trait for the Rust
/// wrapper type around an underlying CoreFoundation type.
///
/// See [`declare_TCFType`] for details.
///
/// [`declare_TCFType`]: macro.declare_TCFType.html
/// [`TCFType`]: base/trait.TCFType.html
#[macro_export]
macro_rules! impl_TCFType {
    ($ty:ident, $ty_ref:ident, $ty_id:ident) => {
        impl_TCFType!($ty<>, $ty_ref, $ty_id);
        unsafe impl $crate::ConcreteCFType for $ty { }
    };

    ($ty:ident<$($p:ident $(: $bound:path)*),*>, $ty_ref:ident, $ty_id:ident) => {
        impl<$($p $(: $bound)*),*> $crate::base::TCFType for $ty<$($p),*> {
            type Ref = $ty_ref;

            #[allow(non_snake_case)]
            #[inline]
            fn as_concrete_TypeRef(&self) -> $ty_ref {
                self.0
            }

            #[inline]
            unsafe fn wrap_under_get_rule(reference: $ty_ref) -> Self {
                assert!(!reference.is_null(), "Attempted to create a NULL object.");
                let reference = $crate::base::CFRetain(reference as *const ::core::ffi::c_void) as $ty_ref;
                $crate::base::TCFType::wrap_under_create_rule(reference)
            }

            #[allow(non_snake_case)]
            #[inline]
            fn as_CFTypeRef(&self) -> $crate::base::CFTypeRef {
                self.as_concrete_TypeRef() as $crate::base::CFTypeRef
            }

            #[inline]
            unsafe fn wrap_under_create_rule(reference: $ty_ref) -> Self {
                assert!(!reference.is_null(), "Attempted to create a NULL object.");
                // we need one PhantomData for each type parameter so call ourselves
                // again with @Phantom $p to produce that
                $ty(reference $(, impl_TCFType!(@Phantom $p))*)
            }

            #[inline]
            fn type_id() -> $crate::base::CFTypeID {
                unsafe {
                    $ty_id()
                }
            }
        }

        #[allow(unused_imports)]
        impl<$($p $(: $bound)*),*> Clone for $ty<$($p),*> {
            #[inline]
            fn clone(&self) -> Self {
                use $crate::base::TCFType;
                unsafe {
                    $ty::wrap_under_get_rule(self.0)
                }
            }
        }

        #[allow(unused_imports)]
        impl<$($p $(: $bound)*),*> PartialEq for $ty<$($p),*> {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                use $crate::base::TCFType;
                self.as_CFType().eq(&other.as_CFType())
            }
        }

        impl<$($p $(: $bound)*),*> Eq for $ty<$($p),*> { }

        #[allow(unused_imports)]
        unsafe impl<'a, $($p $(: $bound)*),*> $crate::base::ToVoid<$ty<$($p),*>> for &'a $ty<$($p),*> {
            fn to_void(&self) -> *const ::core::ffi::c_void {
                use $crate::base::{TCFType, TCFTypeRef};
                self.as_concrete_TypeRef().as_void_ptr()
            }
        }

        #[allow(unused_imports)]
        unsafe impl<$($p $(: $bound)*),*> $crate::base::ToVoid<$ty<$($p),*>> for $ty<$($p),*> {
            fn to_void(&self) -> *const ::core::ffi::c_void {
                use $crate::base::{TCFType, TCFTypeRef};
                self.as_concrete_TypeRef().as_void_ptr()
            }
        }

        #[allow(unused_imports)]
        unsafe impl<$($p $(: $bound)*),*> $crate::base::ToVoid<$ty<$($p),*>> for $ty_ref {
            fn to_void(&self) -> *const ::core::ffi::c_void {
                use $crate::base::TCFTypeRef;
                self.as_void_ptr()
            }
        }

    };

    (@Phantom $x:ident) => { ::std::marker::PhantomData };
}

/// Implement `std::fmt::Debug` for the given type.
///
/// This will invoke the implementation of `Debug` for [`CFType`]
/// which invokes [`CFCopyDescription`].
///
/// The type must have an implementation of the [`TCFType`] trait, usually
/// provided using the [`impl_TCFType`] macro.
///
/// [`CFType`]: base/struct.CFType.html#impl-Debug
/// [`CFCopyDescription`]: https://developer.apple.com/documentation/corefoundation/1521252-cfcopydescription?language=objc
/// [`TCFType`]: base/trait.TCFType.html
/// [`impl_TCFType`]: macro.impl_TCFType.html
#[macro_export]
macro_rules! impl_CFTypeDescription {
    ($ty:ident) => {
        // it's fine to use an empty <> list
        impl_CFTypeDescription!($ty<>);
    };
    ($ty:ident<$($p:ident $(: $bound:path)*),*>) => {
        #[allow(unused_imports)]
        impl<$($p $(: $bound)*),*> ::std::fmt::Debug for $ty<$($p),*> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                use $crate::base::TCFType;
                self.as_CFType().fmt(f)
            }
        }
    }
}

#[macro_export]
macro_rules! impl_CFComparison {
    ($ty:ident, $compare:ident) => {
        impl_CFComparison!($ty<>, $compare);
    };
    ($ty:ident<$($p:ident $(: $bound:path)*),*>, $compare:ident) => {
        impl<$($p $(: $bound)*),*> PartialOrd for $ty<$($p),*> {
            #[inline]
            fn partial_cmp(&self, other: &$ty<$($p),*>) -> Option<::std::cmp::Ordering> {
                unsafe {
                    Some(
                        $compare(
                            self.as_concrete_TypeRef(),
                            other.as_concrete_TypeRef(),
                            ::std::ptr::null_mut(),
                        )
                        .into(),
                    )
                }
            }
        }

        impl<$($p $(: $bound)*),*> Ord for $ty<$($p),*> {
            #[inline]
            fn cmp(&self, other: &$ty<$($p),*>) -> ::std::cmp::Ordering {
                self.partial_cmp(other).unwrap()
            }
        }
    };
}

pub mod array;
pub mod attributed_string;
pub mod base;
pub mod boolean;
pub mod bundle;
pub mod characterset;
pub mod data;
pub mod date;
pub mod dictionary;
pub mod error;
pub mod filedescriptor;
pub mod mach_port;
pub mod number;
pub mod propertylist;
pub mod runloop;
pub mod set;
pub mod string;
pub mod timezone;
pub mod url;
pub mod uuid;
