use core::cell::{RefCell, RefMut};

pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

//Sync trait 通常用于表示类型在多线程环境中是线程安全的，但这里的 UPSafeCell 使用了 RefCell，
//而 RefCell 本身并不是线程安全的。因此，为 UPSafeCell 实现 Sync 是不安全的
unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    //独占访问，获取了可变引用
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}