/*

Glk objects
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::{Arc, Mutex, Weak};

/** Wraps a Glk object in an `Arc<Mutex>`, and ensures they can be used in a HashMap */
#[derive(Default)]
pub struct GlkObject<T> {
    pub obj: Arc<Mutex<T>>,
}

impl<T> GlkObject<T> {
    pub fn new(obj: T) -> Self {
        Self {
            obj: Arc::new(Mutex::new(obj))
        }
    }

    pub fn as_ptr(&self) -> *const Mutex<T> {
        Arc::as_ptr(self)
    }

    pub fn downgrade(&self) -> GlkObjectWeak<T> {
        Arc::downgrade(&self.obj)
    }
}

/** References between objects should use a `Weak` to prevent cycles */
pub type GlkObjectWeak<T> = Weak<Mutex<T>>;

impl<T> Clone for GlkObject<T> {
    fn clone(&self) -> Self {
        GlkObject::<T> {
            obj: self.obj.clone(),
        }
    }
}

impl<T> Deref for GlkObject<T> {
    type Target = Arc<Mutex<T>>;
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
    store: HashMap<GlkObject<T>, GlkObjectMetadata<T>>,
}

pub struct IterationResult<T> {
    pub obj: GlkObject<T>,
    pub rock: u32,
}

impl<T> GlkObjectStore<T>
where T: Default, GlkObject<T>: Eq {
    pub fn new() -> Self {
        GlkObjectStore {
            counter: 1,
            first: None,
            store: HashMap::new(),
        }
    }

    pub fn get_disprock(&self, obj: &GlkObject<T>) -> Option<u32> {
        self.store.get(obj).map(|obj| obj.disprock)
    }

    pub fn get_rock(&self, obj: &GlkObject<T>) -> Option<u32> {
        self.store.get(obj).map(|obj| obj.rock)
    }

    pub fn iterate(&self, obj: Option<&GlkObject<T>>) -> Option<IterationResult<T>> {
        match obj {
            None => self.first.as_ref().map(|weak| weak.into()),
            Some(obj) => self.store.get(obj).unwrap().next(),
        }
        .map(|obj| {
            let rock = self.store.get(&obj).unwrap().rock;
            IterationResult {
                obj,
                rock,
            }
        })
    }

    pub fn register(&mut self, obj: &GlkObject<T>, rock: u32) {
        let mut glk_object = GlkObjectMetadata::new(rock, self.counter);
        self.counter += 1;
        match self.first.as_ref() {
            None => {
                self.first = Some(obj.downgrade());
                self.store.insert(obj.clone(), glk_object);
            },
            Some(old_first) => {
                glk_object.next = Some(old_first.clone());
                let old_first_upgraded = &old_first.into();
                self.store.get_mut(old_first_upgraded).unwrap().prev = Some(obj.downgrade());
                self.first = Some(obj.downgrade());
                self.store.insert(obj.clone(), glk_object);
            }
        };
    }

    /** Remove an object from the store */
    pub fn unregister(&mut self, obj: GlkObject<T>) {
        let glk_obj = self.store.get(&obj).unwrap();
        let prev = glk_obj.prev();
        let next = glk_obj.next();
        if let Some(prev) = &prev {
            self.store.get_mut(prev).unwrap().next = next.as_ref().map(|obj| obj.downgrade());
        }
        if let Some(next) = &next {
            self.store.get_mut(next).unwrap().prev = prev.as_ref().map(|obj| obj.downgrade());
        }
        if let Some(first) = &self.first {
            let first_upgraded: GlkObject<T> = first.into();
            if first_upgraded == obj {
                self.first = None;
            }
        }
        self.store.remove(&obj);
        assert_eq!(Arc::strong_count(&obj), 1, "Dangling strong reference to obj after it was unregistered");
    }
}

impl<T> Default for GlkObjectStore<T>
where T: Default, GlkObject<T>: Eq {
    fn default() -> Self {
        GlkObjectStore::new()
    }
}

/** Contains the private metadata we keep in each object store */
#[derive(Default)]
struct GlkObjectMetadata<T> {
    disprock: u32,
    next: Option<GlkObjectWeak<T>>,
    prev: Option<GlkObjectWeak<T>>,
    rock: u32,
}

impl<T> GlkObjectMetadata<T>
where T: Default {
    fn new(rock: u32, disprock: u32) -> Self {
        GlkObjectMetadata {
            disprock,
            rock,
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