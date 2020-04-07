#![cfg_attr(not(impl_std), no_std)]

#[cfg(impl_std)]
mod impl_std {
    pub struct Mutex<T>(std::sync::Mutex<T>);

    impl<T> Mutex<T> {
        pub fn new(t: T) -> Mutex<T> {
            Mutex(t)
        }
    }

    impl<T> mutex_trait::Mutex for Mutex<T> {
        type Data = T;

        fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
            let mut lock = self.0.lock().expect("lock failed");
            f(&mut *lock)
        }
    }
}

#[cfg(impl_std)]
pub use impl_std::*;

#[cfg(not(impl_std))]
mod impl_cs {
    use core::mem::{ManuallyDrop, MaybeUninit};
    use core::ptr;
    use core::sync::atomic::{AtomicPtr, Ordering::SeqCst};

    type CriticalSectionFunc = fn(ctx: *mut (), f: fn(ctx: *mut ()) -> ());

    // This could be done with a weak symbol but unfortunately Rust doesn't do those yet.
    static THE_CRITICAL_SECTION: AtomicPtr<CriticalSectionFunc> = AtomicPtr::new(ptr::null_mut());

    pub struct Mutex<T>(T);

    impl<T> Mutex<T> {
        pub fn new(t: T) -> Mutex<T> {
            Mutex(t)
        }

        fn lock_impl<R, F: FnOnce(&mut T) -> R>(&mut self, f: F) -> R {
            unsafe {
                let mut cs_ptr: *const CriticalSectionFunc = THE_CRITICAL_SECTION.load(SeqCst);

                if cs_ptr.is_null() {
                    cs_ptr = &the_critical_section as *const _ as _;
                }

                let mut ctx = (
                    &mut self.0,
                    ManuallyDrop::new(f),
                    MaybeUninit::<R>::uninit());

                (&*cs_ptr)(&mut ctx as *mut _ as *mut (), |ctx_ptr| {
                    let ctx = &mut *(ctx_ptr as *mut (
                        &mut T,
                        ManuallyDrop<F>,
                        MaybeUninit<R>));

                    let f = ManuallyDrop::take(&mut ctx.1);

                    let r = f(ctx.0);

                    ptr::write(ctx.2.as_mut_ptr(), r);
                });

                ctx.2.assume_init()
            }
        }
    }

    impl<T> mutex_trait::Mutex for Mutex<T> {
        type Data = T;

        fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
            self.lock_impl(f)
        }
    }

    #[cfg(impl_cortex_m)]
    fn the_critical_section(ctx: *mut (), f: fn(ctx: *mut ()) -> ()) {
        cortex_m::interrupt::free(|_| f(ctx))
    }

    #[cfg(impl_riscv)]
    fn the_critical_section(ctx: *mut (), f: fn(ctx: *mut ()) -> ()) {
        riscv::interrupt::free(|_| f(ctx))
    }

    #[cfg(not(any(impl_cortex_m, impl_riscv)))]
    fn the_critical_section(_ctx: *mut (), _f: fn(ctx: *mut ()) -> ()) {
        panic!("not supported");
    }

    pub unsafe fn set_the_critical_section(f: *const CriticalSectionFunc) {
        THE_CRITICAL_SECTION.store(f as *mut _, SeqCst);
    }
}

#[cfg(not(impl_std))]
pub use impl_cs::*;
