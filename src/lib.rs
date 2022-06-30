#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

/// Associates a static object of type T and a marker TAG.
/// Use the `assoc_threadlocal!()` macro for implementing this trait on types.
pub trait AssocThreadLocal<T: Copy, TAG = ()> {
    /// Returns the associated thread local object of the Self type
    ///
    /// # Safety
    /// The returned pointer must be immediately used, not stored/passed somewhere else.
    unsafe fn the_threadlocal() -> *const std::cell::Cell<T>;

    /// Returns the associated thread local object of the Self type
    fn get_threadlocal() -> T {
        unsafe { (*Self::the_threadlocal()).get() }
    }

    /// Sets the associated thread local object of the Self type
    fn set_threadlocal(value: T) {
        unsafe {
            (*Self::the_threadlocal()).set(value);
        }
    }

    /// Returns the associated threadlocal object from an instance.
    fn get_threadlocal_from(_this: &Self) -> T {
        Self::get_threadlocal()
    }

    /// Sets the associated threadlocal object from an instance.
    fn set_threadlocal_of(_this: &Self, value: T) {
        Self::set_threadlocal(value)
    }
}

/// Helper macro doing the boilerplate implementation.
/// This must be a macro because we can not use generic parameters from the outer scope.
///
///  * 'TAG' A type marker to discriminate this implementation, defaults to ()
///  * 'T' is the type you want have a thread local object associated to
///  * 'TARGET' is the type of the thread local object
///  * 'INIT' is used to initialize the thread local object
///
/// The simple case, associate something to some local type:
/// ```
/// use crate::assoc_threadlocal::*;
///
/// // define a type and attach a '&str' object to it
/// struct Example;
/// assoc_threadlocal!(Example, &'static str = "&str associated to Example");
///
/// // get it by type
/// assert_eq!(Example::get_threadlocal(), "&str associated to Example");
///
/// // get it from an object
/// let example = Example;
/// assert_eq!(AssocThreadLocal::get_threadlocal_from(&example), "&str associated to Example");
/// ```
///
/// The 'TAG' is required when one needs to disambiguate between different target values of
/// the same type or when an association between foreign types not defined in the current
/// crate shall be established. This can be any (non-generic) type your crate defines,
/// preferably you just make a zero-size struct just for this purpose. It is only used as
/// marker for disambiguation.
///
/// Disambiguate between different thread local objects:
/// ```
/// use crate::assoc_threadlocal::*;
///
/// struct Example;
///
/// // attach a '&str' object to Example
/// struct Hello;
/// assoc_threadlocal!(Hello:Example, &'static str = "Hello World!");
///
/// // again but for another purpose
/// struct ExplainType;
/// assoc_threadlocal!(ExplainType:Example, &'static str = "This is 'struct Example'");
///
/// let example = Example;
///
/// // resolve the ambiguity with a turbofish
/// assert_eq!(AssocThreadLocal::<_, Hello>::get_threadlocal_from(&example), "Hello World!");
/// assert_eq!(AssocThreadLocal::<_, ExplainType>::get_threadlocal_from(&example), "This is 'struct Example'");
/// ```
///
/// Make an association between foreign types:
/// ```
/// use crate::assoc_threadlocal::*;
///
/// // attach a '&str' to i32
/// struct I32ExampleStr;
/// assoc_threadlocal!(I32ExampleStr:i32, &'static str = "&str associated to i32");
///
/// // get it
/// assert_eq!(AssocThreadLocal::get_threadlocal_from(&100i32), "&str associated to i32");
/// ```
#[macro_export]
macro_rules! assoc_threadlocal {
    ($TAG:ty:$T:ty, $TARGET:ty = $INIT:expr) => {
        impl $crate::AssocThreadLocal<$TARGET, $TAG> for $T {
            unsafe fn the_threadlocal() -> *const std::cell::Cell<$TARGET> {
                std::thread_local!(
                    static ASSOCIATED_THREADLOCAL: (
                        std::cell::Cell<$TARGET>,
                        std::marker::PhantomData<$crate::MakeSync<$T>>,
                        std::marker::PhantomData<$crate::MakeSync<$TAG>>,
                    ) = (
                        std::cell::Cell::new($INIT),
                        std::marker::PhantomData,
                        std::marker::PhantomData,
                    );
                );
                ASSOCIATED_THREADLOCAL.with(|l| &l.0 as *const std::cell::Cell<$TARGET>)
            }
        }
    };
    ($T:ty, $TARGET:ty = $INIT:expr) => {
        impl $crate::AssocThreadLocal<$TARGET, ()> for $T {
            unsafe fn the_threadlocal() -> *const std::cell::Cell<$TARGET> {
                std::thread_local!(
                    static ASSOCIATED_THREADLOCAL: (
                        std::cell::Cell<$TARGET>,
                        std::marker::PhantomData<$crate::MakeSync<$T>>,
                        std::marker::PhantomData<$crate::MakeSync<()>>,
                    ) = (
                        std::cell::Cell::new($INIT),
                        std::marker::PhantomData,
                        std::marker::PhantomData,
                    );
                );
                ASSOCIATED_THREADLOCAL.with(|l| &l.0 as *const std::cell::Cell<$TARGET>)
            }
        }
    };
}

/// Only a helper, needs to be public because of the macro
#[doc(hidden)]
pub struct MakeSync<T>(T);
unsafe impl<T> Sync for MakeSync<T> {}

#[cfg(test)]
mod tests {
    use crate::AssocThreadLocal;

    struct TestType1;
    assoc_threadlocal!(TestType1, &'static str = "This is the first test type");

    #[test]
    fn get_threadlocal() {
        assert_eq!(TestType1::get_threadlocal(), "This is the first test type");
    }

    #[test]
    fn set_threadlocal() {
        TestType1::set_threadlocal("This is the first test type, set to a new value");
        assert_eq!(
            TestType1::get_threadlocal(),
            "This is the first test type, set to a new value"
        );
    }

    struct TestType2;
    assoc_threadlocal!(TestType2, &'static str = "This is the second test type");
    assoc_threadlocal!(TestType2, u32 = 42);

    #[test]
    fn multiple_threadlocals() {
        assert_eq!(
            <TestType2 as AssocThreadLocal<&str, ()>>::get_threadlocal(),
            "This is the second test type"
        );
        assert_eq!(
            <TestType2 as AssocThreadLocal<u32, ()>>::get_threadlocal(),
            42
        );
    }

    #[test]
    fn from_instance() {
        let test = TestType1;
        assert_eq!(
            AssocThreadLocal::get_threadlocal_from(&test),
            "This is the first test type"
        );
    }

    #[test]
    fn from_instance_multiple() {
        let test = TestType2;
        assert_eq!(
            AssocThreadLocal::<&str, _>::get_threadlocal_from(&test),
            "This is the second test type"
        );
        assert_eq!(AssocThreadLocal::<u32, _>::get_threadlocal_from(&test), 42);
    }
}
