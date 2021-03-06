use std::collections::HashSet;
use std::sync::RwLock;
use std::ops::Deref;
use std::marker::PhantomData;

/// A thread-safe set of references to items owned externally by an ItemOwner.
///
/// When an ItemOwner is dropped or its .take() method is called, the
/// item is removed from the collection.
pub struct ExternalSet<T> {
    items: RwLock<HashSet<*mut T>>
}

impl<T> ExternalSet<T> {
    /// Create an empty ExternalSet
    pub fn new() -> ExternalSet<T> {
        ExternalSet { items: RwLock::new(HashSet::new()) }
    }

    /// Add an item to the collection, returning an ItemOwner to own it.
    /// When the ItemOwner is dropped, the value will be removed from the collection.
    pub fn insert<'s>(&'s self, item: T) -> ItemOwner<'s, T> {
        let ptr = Box::into_raw(Box::new(item));
        self.items.write().unwrap().insert(ptr);
        ItemOwner { collection: self, ptr: ptr, _marker: PhantomData }
    }

    /// Lock the collection for iteration. References obtained from the iterator have the
    /// lifetime of the returned guard.
    pub fn lock(&self) -> ExternalSetReadGuard<T> {
        ExternalSetReadGuard(self.items.read().unwrap())
    }
}

unsafe impl<T: Sync> Sync for ExternalSet<T> {}
unsafe impl<T: Send + Sync> Send for ExternalSet<T> {}

/// RAII structure used to iterate over the items in a ExternalSet, and unlock the collection
/// when dropped.
pub struct ExternalSetReadGuard<'c, T: 'c>(::std::sync::RwLockReadGuard<'c, HashSet<*mut T>>);

impl<'c, T> ExternalSetReadGuard<'c, T> {
    /// Iterate over references to items in the ExternalSet.
    ///
    /// The order of iteration is unspecified.
    pub fn iter<'g: 'c>(&'g self) -> ExternalSetIter<'g, T> {
        ExternalSetIter { iter: self.0.iter(), except: None, }
    }

    /// Iterate over references to items in the ExternalSet, excluding the one owned by the
    /// provided ItemOwner.
    ///
    /// If the provided ItemOwner is not in the collection (e.g. it was inserted into another
    /// collection), all items will be yielded by the iterator.
    ///
    /// The order of iteration is unspecified.
    pub fn others<'g: 'c>(&'g self, except: &ItemOwner<T>) -> ExternalSetIter<'g, T> {
        ExternalSetIter { iter: self.0.iter(), except: Some(except.ptr) }
    }
}

/// Iterator over the items in a ExternalSet.
pub struct ExternalSetIter<'g, T: 'g> {
    iter: ::std::collections::hash_set::Iter<'g, *mut T>,
    except: Option<*mut T>
}

impl<'g, T> Iterator for ExternalSetIter<'g, T> {
    type Item = &'g T;

    fn next(&mut self) -> Option<&'g T> {
        if let Some(&i) = self.iter.next() {
            if Some(i) == self.except { return self.next(); } // skip excluded item
            Some(unsafe { &*i })
        } else {
            None
        }
    }
}

/// The owner of an item in a ExternalSet
pub struct ItemOwner<'s, T: 's> {
    collection: &'s ExternalSet<T>,
    ptr: *mut T,
    _marker: PhantomData<T>,
}

impl<'s, T> ItemOwner<'s, T> {
    /// Remove the item from the collection and return it.
    pub fn take(self) -> T {
        unsafe {
            self.collection.items.write().unwrap().remove(&self.ptr);
            let ret = *Box::from_raw(self.ptr);
            ::std::mem::forget(self);
            ret
        }
    }
}

unsafe impl<'s, T: Send + Sync> Send for ItemOwner<'s, T> {}
unsafe impl<'s, T: Sync> Sync for ItemOwner<'s, T> {}

impl<'s, T> Drop for ItemOwner<'s, T> {
    fn drop(&mut self) {
        self.collection.items.write().unwrap().remove(&self.ptr);
        drop(unsafe { *Box::from_raw(self.ptr) });
    }
}

impl <'s, T> Deref for ItemOwner<'s, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

#[test]
fn test1() {
    let c = ExternalSet::<u32>::new();
    let i1 = c.insert(1);
    let i2 = c.insert(2);
    let i3 = c.insert(3);

    let mut items = c.lock().iter().map(|&i| i).collect::<Vec<_>>();
    items.sort();
    assert_eq!(items, vec![1, 2, 3]);

    let mut items = c.lock().others(&i1).map(|&i| i).collect::<Vec<_>>();
    items.sort();
    assert_eq!(items, vec![2, 3]);

    assert_eq!(i2.take(), 2);

    let mut items = c.lock().iter().map(|&i| i).collect::<Vec<_>>();
    items.sort();
    assert_eq!(items, vec![1, 3]);

    assert_eq!(*i1, 1);

    drop(i1);
    drop(i3);

    assert_eq!(c.lock().iter().count(), 0);
}
