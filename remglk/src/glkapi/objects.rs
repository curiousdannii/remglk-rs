/*

Glk objects & dispatch
======================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, Weak};

/** Wraps a Glk object in an `Arc<Mutex>`, and ensures they can be hashed */
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

    pub fn as_ptr(&self) -> *const Mutex<GlkObjectMetadata<T>> {
        Arc::as_ptr(self)
    }

    pub fn downgrade(&self) -> GlkObjectWeak<T> {
        Arc::downgrade(&self.obj)
    }
}

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

// Hash GlkObjects by the Arc's pointer
impl<T> Hash for GlkObject<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.obj).hash(state);
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
    store: HashSet<GlkObject<T>>,
    unregister_cb: Option<DispatchUnregisterCallback<T>>,
}

impl<T> GlkObjectStore<T>
where T: Default + GlkObjectClass, GlkObject<T>: Default + Eq {
    pub fn new() -> Self {
        GlkObjectStore {
            counter: 1,
            object_class: T::get_object_class_id(),
            store: HashSet::new(),
            ..Default::default()
        }
    }

    pub fn iterate(&self, obj: Option<&GlkObject<T>>) -> Option<GlkObject<T>> {
        let next_weak = match obj {
            Some(obj) => obj.lock().unwrap().next.as_ref().map(|weak| weak.clone()),
            None => self.first.clone(),
        };
        next_weak.map(|obj| (&obj).into())
    }

    pub fn register(&mut self, obj_glkobj: &GlkObject<T>, rock: u32) {
        let obj_ptr = obj_glkobj.as_ptr();
        let mut obj = obj_glkobj.lock().unwrap();
        obj.id = self.counter;
        obj.rock = rock;
        self.counter += 1;
        if let Some(register_cb) = self.register_cb {
            obj.disprock = Some(register_cb(obj_ptr, self.object_class));
        }
        match self.first.as_ref() {
            None => {
                self.first = Some(obj_glkobj.downgrade());
                self.store.insert(obj_glkobj.clone());
            },
            Some(old_first) => {
                obj.next = Some(old_first.clone());
                let old_first: GlkObject<T> = old_first.into();
                let mut old_first = old_first.lock().unwrap();
                old_first.prev = Some(obj_glkobj.downgrade());
                self.first = Some(obj_glkobj.downgrade());
                self.store.insert(obj_glkobj.clone());
            }
        };
    }

    pub fn set_callbacks(&mut self, register_cb: DispatchRegisterCallback<T>, unregister_cb: DispatchUnregisterCallback<T>) {
        self.register_cb = Some(register_cb);
        self.unregister_cb = Some(unregister_cb);
        for obj in self.store.iter() {
            let obj_ptr = obj.as_ptr();
            let mut obj = obj.lock().unwrap();
            obj.disprock = Some(register_cb(obj_ptr, self.object_class));
        }
    }

    /** Remove an object from the store */
    pub fn unregister(&mut self, obj_glkobj: GlkObject<T>) {
        let obj_ptr = obj_glkobj.as_ptr();
        let mut obj = obj_glkobj.lock().unwrap();
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
        if let Some(unregister_cb) = self.unregister_cb {
            let disprock = obj.disprock.take().unwrap();
            unregister_cb(obj_ptr, self.object_class, disprock);
        }
        self.store.remove(&obj_glkobj);
        assert_eq!(Arc::strong_count(&obj_glkobj), 1, "Dangling strong reference to obj after it was unregistered");
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
    pub disprock: Option<DispatchRock>,
    /** The ID, used in the GlkOte protocol */
    pub id: u32,
    obj: T,
    next: Option<GlkObjectWeak<T>>,
    prev: Option<GlkObjectWeak<T>>,
    pub rock: u32,
}

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
#[repr(C)]
pub struct DispatchRock {
    _data: [u8; 0],
    // Not thread safe, not sure if it's okay without it though
    //_marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

pub type DispatchRegisterCallback<T> = fn(*const Mutex<GlkObjectMetadata<T>>, u32) -> DispatchRock;
pub type DispatchUnregisterCallback<T> = fn(*const Mutex<GlkObjectMetadata<T>>, u32, DispatchRock);

pub trait GlkObjectClass {
    fn get_object_class_id() -> u32;
}