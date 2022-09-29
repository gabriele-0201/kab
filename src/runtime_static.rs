/*
#[macro_export]
macro_rules! lazy_static {
    (($($vis:tt)*) static $name:ident : $type:ty = $value:expr) => {

        $($vis)* struct $name {__uless_field: ()}


        impl core::ops::Deref for $name {
            type Target = $type;

            fn deref(&self) -> &Self::Target {

                fn __static_fn() -> $type { $value }
                
                static $name : FnOnce() -> $type = __static_fn();



            }

        }

        $($vis)* static $name : $name = $name {__uless_field: ()};
    }
}
*/

use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{ AtomicBool, Ordering }
};

// Use this to allow the creation of something static but
// depends on runtime
pub struct RuntimeStatic<T> {  
     init: AtomicBool,
     data: UnsafeCell<MaybeUninit<T>>
}

unsafe impl<T> Sync for RuntimeStatic<T> {}

impl<T> RuntimeStatic<T> {
    pub const fn get_uninit() -> Self {
        Self {
            init: AtomicBool::new(false), // false
            data: UnsafeCell::new(MaybeUninit::uninit())
        }
    }

    pub fn init(&self, data: T) {
        if self.init.swap(true, Ordering::Relaxed) {
            panic!("RuntimeStatic already init");
        }

        let data_container = unsafe{ &mut *self.data.get() };
        data_container.write(data);
    }
}

impl<T> core::ops::Deref for RuntimeStatic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target{
        if !self.init.load(Ordering::Relaxed) {
            panic!("Impossible dereference ad Unint data");
        }

        unsafe {
            let data_container = &mut *self.data.get();
            data_container.assume_init_ref()
        }
    }
}

impl<T> core::ops::DerefMut for RuntimeStatic<T> {

    fn deref_mut(&mut self) -> &mut Self::Target{
        if !self.init.load(Ordering::Relaxed) {
            panic!("Impossible dereference ad Unint data");
        }

        unsafe {
            let data_container = &mut *self.data.get();
            data_container.assume_init_mut()
        }
    }
}
