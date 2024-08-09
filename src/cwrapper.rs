use derive_more::Deref;
use std::ffi::c_void;
use std::mem::MaybeUninit;

#[derive(Deref)]
pub struct CArrayWrapper<T> {
    #[deref]
    wrapper: CWrapper<T>,
    len: usize,
}

impl<T> CArrayWrapper<T> {
    pub fn new(ptr: *mut T, len: usize) -> Option<Self> {
        CWrapper::new(ptr).map(|wrapper| Self { wrapper, len })
    }

    pub fn new_with_destroyer(
        ptr: *mut T,
        len: usize,
        destroy: unsafe extern "C" fn(*mut T),
    ) -> Option<Self> {
        CWrapper::new_with_destroyer(ptr, destroy).map(|wrapper| Self { wrapper, len })
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.wrapper.as_ptr(), self.len) }
    }
}

impl<T> AsRef<[T]> for CArrayWrapper<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for CArrayWrapper<T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.wrapper.as_mut_ptr(), self.len) }
    }
}

#[derive(Deref)]
pub struct OwnedCWrapper<T> {
    #[deref]
    data: T,
    destroy: unsafe extern "C" fn(*mut T),
}

impl<T> OwnedCWrapper<T> {
    pub fn new(ptr: T, destroy: unsafe extern "C" fn(*mut T)) -> Self {
        Self { data: ptr, destroy }
    }
}

impl<T> Drop for OwnedCWrapper<T> {
    fn drop(&mut self) {
        unsafe {
            (self.destroy)(&mut self.data);
        }
    }
}

pub struct CWrapper<T> {
    ptr: *mut T,
    destroy: Option<unsafe extern "C" fn(*mut T)>,
}

unsafe impl<T> Send for CWrapper<T> {}
unsafe impl<T> Sync for CWrapper<T> {}

impl<T> CWrapper<T> {
    pub fn new(ptr: *mut T) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr, destroy: None })
        }
    }

    pub fn new_with_destroyer(ptr: *mut T, destroy: unsafe extern "C" fn(*mut T)) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ptr,
                destroy: Some(destroy),
            })
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }
}

impl<T> AsRef<T> for CWrapper<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> AsMut<T> for CWrapper<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Deref for CWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> Drop for CWrapper<T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(destroy) = self.destroy {
                destroy(self.ptr);
            } else {
                crate::bindings::free(self.ptr as *mut c_void);
            }
        }
    }
}

pub struct CIteratorWrapper<T, ItemT> {
    iterator: CWrapper<T>,
    done: unsafe extern "C" fn(*mut T, *mut ItemT) -> bool,
    advance: unsafe extern "C" fn(*mut T),
}

impl<T: 'static, ItemT: 'static> CIteratorWrapper<T, ItemT> {
    pub fn new(
        raw_iterator: *mut T,
        destroy: unsafe extern "C" fn(*mut T),
        done: unsafe extern "C" fn(*mut T, *mut ItemT) -> bool,
        advance: unsafe extern "C" fn(*mut T),
    ) -> impl Iterator<Item = ItemT> + 'static {
        CWrapper::new_with_destroyer(raw_iterator, destroy)
            .into_iter()
            .flat_map(move |iterator| Self {
                iterator,
                done,
                advance,
            })
    }
}

impl<T, ItemT> Iterator for CIteratorWrapper<T, ItemT> {
    type Item = ItemT;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut item = MaybeUninit::uninit().assume_init();
            if (self.done)(self.iterator.as_mut_ptr(), &mut item) {
                None
            } else {
                (self.advance)(self.iterator.as_mut_ptr());
                Some(item)
            }
        }
    }
}
