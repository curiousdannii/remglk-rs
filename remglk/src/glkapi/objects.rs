/*

Glk objects
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::num::NonZeroU32;

/** A store for Glk objects of a particular type.
    The object store will own the objects.
 */
// TODO: Can we use a new type here?
pub struct GlkObjectStore<T> {
    counter: NonZeroU32,
    first: Option<NonZeroU32>,
    store: HashMap<NonZeroU32, GlkObject<T>>,
}

pub struct IterationResult {
    pub id: NonZeroU32,
    pub rock: u32,
}

impl<T> GlkObjectStore<T> {
    pub fn new() -> Self {
        GlkObjectStore {
            counter: NonZeroU32::new(1).unwrap(),
            first: None,
            store: HashMap::new(),
        }
    }

    pub fn get(&self, id: Option<NonZeroU32>) -> Option<&T> {
        self.store.get(&id?).map(|obj| &obj.obj)
    }

    pub fn get_mut(&mut self, id: Option<NonZeroU32>) -> Option<&mut T> {
        self.store.get_mut(&id?).map(|obj| &mut obj.obj)
    }

    pub fn get_rock(&self, id: Option<NonZeroU32>) -> Option<u32> {
        self.store.get(&id?).map(|obj| obj.rock)
    }

    pub fn iterate(&self, id: Option<NonZeroU32>) -> Option<IterationResult> {
        let next = match id {
            None => self.first,
            Some(id) => self.store.get(&id).unwrap().next,
        };
        next.map(|id| IterationResult {
            id,
            rock: self.store.get(&id).unwrap().rock,
        })
    }

    pub fn register(&mut self, obj: T, rock: u32) -> NonZeroU32 {
        let new_id = self.counter;
        self.counter = self.counter.checked_add(1).expect("ran out of object IDs!");
        let mut glk_object = GlkObject::new(obj, rock);
        match self.first {
            None => {
                self.store.insert(new_id, glk_object);
                self.first = Some(new_id);
            },
            Some(old_first_id) => {
                self.store.get_mut(&old_first_id).unwrap().prev = Some(new_id);
                glk_object.next = Some(old_first_id);
                self.store.insert(new_id, glk_object);
                self.first = Some(new_id);
            }
        };
        new_id
    }

    /** Remove an object from the store */
    pub fn unregister(&mut self, id: NonZeroU32) {
        let prev = self.store.get_mut(&id).unwrap().prev;
        let next = self.store.get_mut(&id).unwrap().next;
        if let Some(prev_id) = prev {
            self.store.get_mut(&prev_id).unwrap().next = next;
        }
        if let Some(next_id) = next {
            self.store.get_mut(&next_id).unwrap().prev = prev;
        }
        if let Some(first_id) = self.first {
            if first_id == id {
                self.first = None;
            }
        }
        self.store.remove(&id);
    }
}

/** Contains the private data we keep in each object store */
struct GlkObject<T> {
    _disprock: Option<u32>,
    next: Option<NonZeroU32>,
    obj: T,
    prev: Option<NonZeroU32>,
    rock: u32,
}

impl<T> GlkObject<T> {
    fn new(obj: T, rock: u32) -> Self {
        GlkObject {
            _disprock: None,
            next: None,
            obj,
            prev: None,
            rock,
        }
    }
}