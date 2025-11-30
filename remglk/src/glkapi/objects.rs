/*

Glk objects & dispatch
======================

Copyright (c) 2025 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

/** Wraps a Glk object in an `Arc<Mutex>` */
#[derive(Default)]
pub struct GlkObject<T> {
    pub obj: Arc<Mutex<GlkObjectMetadata<T>>>,
}

impl<T> GlkObject<T>
where T: Default {
    pub fn new(obj: T) -> Self {
        Self {
            obj: Arc::new(Mutex::new(GlkObjectMetadata::new(obj)))
        }
    }

    pub fn new_cyclic<F>(data_fn: F) -> Self
    where F: FnOnce(&GlkObjectWeak<T>) -> T {
        Self {
            obj: Arc::new_cyclic(|weak| {
                Mutex::new(GlkObjectMetadata::new(data_fn(weak)))
            })
        }
    }

    pub fn as_ptr(&self) -> *const Mutex<GlkObjectMetadata<T>> {
        Arc::as_ptr(self)
    }

    pub fn downgrade(&self) -> GlkObjectWeak<T> {
        Arc::downgrade(&self.obj)
    }
}

pub type LockedGlkObject<'a, T> = MutexGuard<'a, GlkObjectMetadata<T>>;

/** References between objects should use a `Weak` to prevent cycles */
pub type GlkObjectWeak<T> = Weak<Mutex<GlkObjectMetadata<T>>>;

impl<T> Clone for GlkObject<T> {
    fn clone(&self) -> Self {
        GlkObject::<T> {
            obj: self.obj.clone(),
        }
    }
}

impl<T> Deref for GlkObject<T> {
    type Target = Arc<Mutex<GlkObjectMetadata<T>>>;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl<T> From<&GlkObjectWeak<T>> for GlkObject<T> {
    fn from(weak: &GlkObjectWeak<T>) -> Self {
        GlkObject {
            obj: weak.upgrade().unwrap(),
        }
    }
}

// Two GlkObject are equal if they point to the same object
impl<T> PartialEq for GlkObject<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.obj, &other.obj)
    }
}
impl<T> Eq for GlkObject<T> {}

/** A metadata store for Glk objects of a particular type. */
pub struct GlkObjectStore<T> {
    counter: u32,
    first: Option<GlkObjectWeak<T>>,
    object_class: u32,
    register_cb: Option<DispatchRegisterCallback<T>>,
    store: HashMap<u32, GlkObject<T>>,
    unregister_cb: Option<DispatchUnregisterCallback<T>>,
}

impl<T> GlkObjectStore<T>
where T: Default + GlkObjectClass, GlkObject<T>: Default + Eq {
    pub fn new() -> Self {
        GlkObjectStore {
            counter: 1,
            first: None,
            object_class: T::get_object_class_id(),
            register_cb: None,
            store: HashMap::new(),
            unregister_cb: None,
        }
    }

    pub fn get_by_id(&self, id: u32) -> Option<GlkObject<T>> {
        self.store.get(&id).map(|obj| obj.clone())
    }

    pub fn iter(&self) -> impl Iterator<Item=&GlkObject<T>>{
        self.store.values()
    }

    pub fn iterate(&self, obj: Option<&GlkObject<T>>) -> Option<GlkObject<T>> {
        let next_weak = match obj {
            Some(obj) => obj.lock().unwrap().next.as_ref().cloned(),
            None => self.first.clone(),
        };
        next_weak.map(|obj| (&obj).into())
    }

    pub fn next_id(&self) -> u32 {
        self.counter
    }

    pub fn register(&mut self, obj_glkobj: &GlkObject<T>, rock: u32) {
        let obj_ptr = obj_glkobj.as_ptr();
        let mut obj = obj_glkobj.lock().unwrap();
        let id = self.counter;
        self.counter += 1;
        obj.id = id;
        obj.rock = rock;
        if let Some(register_cb) = self.register_cb {
            obj.disprock = Some(register_cb(obj_ptr, self.object_class));
        }
        match self.first.as_ref() {
            None => {
                self.first = Some(obj_glkobj.downgrade());
                self.store.insert(id, obj_glkobj.clone());
            },
            Some(old_first) => {
                obj.next = Some(old_first.clone());
                let old_first: GlkObject<T> = old_first.into();
                let mut old_first = old_first.lock().unwrap();
                old_first.prev = Some(obj_glkobj.downgrade());
                self.first = Some(obj_glkobj.downgrade());
                self.store.insert(id, obj_glkobj.clone());
            }
        };
    }

    pub fn set_callbacks(&mut self, register_cb: DispatchRegisterCallback<T>, unregister_cb: DispatchUnregisterCallback<T>) {
        self.register_cb = Some(register_cb);
        self.unregister_cb = Some(unregister_cb);
        for obj in self.store.values() {
            let obj_ptr = obj.as_ptr();
            let mut obj = obj.lock().unwrap();
            obj.disprock = Some(register_cb(obj_ptr, self.object_class));
        }
    }

    /** Remove an object from the store */
    pub fn unregister(&mut self, obj_glkobj: GlkObject<T>) {
        let obj_ptr = obj_glkobj.as_ptr();
        let obj = obj_glkobj.lock().unwrap();
        let id = obj.id;
        let prev = obj.prev();
        let next = obj.next();
        if let Some(prev) = &prev {
            let mut prev = prev.lock().unwrap();
            prev.next = next.as_ref().map(|obj| obj.downgrade());
        }
        if let Some(next) = &next {
            let mut next = next.lock().unwrap();
            next.prev = prev.as_ref().map(|obj| obj.downgrade());
        }
        if let Some(first) = &self.first {
            let first: GlkObject<T> = first.into();
            if first == obj_glkobj {
                self.first = next.as_ref().map(|obj| obj.downgrade());
            }
        }
        if let Some(disprock) = obj.disprock {
            drop(obj);
            self.unregister_cb.unwrap()(obj_ptr, self.object_class, disprock);
        }
        self.store.remove(&id).unwrap();
        assert!(Arc::strong_count(&obj_glkobj) == 1, "Dangling strong reference to obj (ptr {:?}, class {}) after it was unregistered", obj_glkobj.as_ptr(), self.object_class);
    }
}

impl<T> Default for GlkObjectStore<T>
where T: Default + GlkObjectClass, GlkObject<T>: Default + Eq {
    fn default() -> Self {
        GlkObjectStore::new()
    }
}

/** Contains the private metadata we keep in each object store */
#[derive(Default)]
pub struct GlkObjectMetadata<T> {
    pub array_disprock: Option<DispatchRock>,
    pub disprock: Option<DispatchRock>,
    /** The ID, used in the GlkOte protocol */
    pub id: u32,
    obj: T,
    next: Option<GlkObjectWeak<T>>,
    prev: Option<GlkObjectWeak<T>>,
    pub rock: u32,
}
unsafe impl<T> Send for GlkObjectMetadata<T> {}

impl<T> GlkObjectMetadata<T>
where T: Default {
    fn new(obj: T) -> Self {
        GlkObjectMetadata {
            obj,
            ..Default::default()
        }
    }

    fn next(&self) -> Option<GlkObject<T>> {
        self.next.as_ref().map(|weak| weak.into())
    }

    fn prev(&self) -> Option<GlkObject<T>> {
        self.prev.as_ref().map(|weak| weak.into())
    }
}

impl<T> Deref for GlkObjectMetadata<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl<T> DerefMut for GlkObjectMetadata<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.obj
    }
}

/** A dispatch rock, which could be anything (*not* the same as a normal Glk rock) */
// See for explanation https://stackoverflow.com/a/38315613/2854284
#[repr(C)]
pub struct DispatchRockPtr {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union DispatchRock {
    num: u32,
    ptr: *const DispatchRockPtr,
    // Add a u64 dummy variant to work around https://github.com/rust-lang/rust/issues/121408
    dummy_variant: u64,
}

pub type DispatchRegisterCallback<T> = fn(*const Mutex<GlkObjectMetadata<T>>, u32) -> DispatchRock;
pub type DispatchUnregisterCallback<T> = fn(*const Mutex<GlkObjectMetadata<T>>, u32, DispatchRock);

pub trait GlkObjectClass {
    fn get_object_class_id() -> u32;
}