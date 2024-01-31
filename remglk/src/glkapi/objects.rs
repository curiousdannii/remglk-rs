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
use std::sync::{Arc, Mutex};

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

    pub fn to_owned(self) -> *const Mutex<T> {
        Arc::into_raw(self.obj)
    }
}

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
    first: Option<GlkObject<T>>,
    store: HashMap<GlkObject<T>, GlkObjectMetadata<T>>,
}

pub struct IterationResult<'a, T> {
    pub obj: &'a GlkObject<T>,
    pub rock: u32,
}

impl<T> GlkObjectStore<T>
where T: Default, GlkObject<T>: Eq {
    pub fn new() -> Self {
        GlkObjectStore {
            first: None,
            store: HashMap::new(),
        }
    }

    pub fn get_rock(&self, obj: &GlkObject<T>) -> Option<u32> {
        self.store.get(obj).map(|obj| obj.rock)
    }

    pub fn iterate(&self, obj: Option<&GlkObject<T>>) -> Option<IterationResult<T>> {
        match obj {
            None => self.first.as_ref(),
            Some(obj) => self.store.get(obj).unwrap().next.as_ref(),
        }
        .map(|obj| IterationResult {
            obj,
            rock: self.store.get(obj).unwrap().rock,
        })
    }

    pub fn register(&mut self, obj: &GlkObject<T>, rock: u32) {
        let mut glk_object = GlkObjectMetadata::new(rock);
        match self.first.as_ref() {
            None => {
                self.first = Some(obj.clone());
                self.store.insert(obj.clone(), glk_object);
            },
            Some(old_first) => {
                self.store.get_mut(old_first).unwrap().prev = Some(obj.clone());
                glk_object.next = Some(old_first.clone());
                self.first = Some(obj.clone());
                self.store.insert(obj.clone(), glk_object);
            }
        };
    }

    /** Remove an object from the store */
    pub fn unregister(&mut self, obj: GlkObject<T>) {
        let glk_obj = self.store.get(&obj).unwrap();
        let prev = glk_obj.prev.as_ref().cloned();
        let next = glk_obj.next.as_ref().cloned();
        if let Some(prev) = &prev {
            self.store.get_mut(prev).unwrap().next = next.as_ref().cloned();
        }
        if let Some(next) = &next {
            self.store.get_mut(next).unwrap().prev = prev.as_ref().cloned();
        }
        if let Some(first) = &self.first {
            if first == &obj {
                self.first = None;
            }
        }
        self.store.remove(&obj);
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
    disprock: Option<u32>,
    next: Option<GlkObject<T>>,
    prev: Option<GlkObject<T>>,
    rock: u32,
}

impl<T> GlkObjectMetadata<T>
where T: Default {
    fn new(rock: u32) -> Self {
        GlkObjectMetadata {
            rock,
            ..Default::default()
        }
    }
}