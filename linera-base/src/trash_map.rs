// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

use std::{
    hash::Hash,
    ops::{Deref, DerefMut},
};

use dashmap::{
    mapref::{
        entry::Entry,
        one::{MappedRef, MappedRefMut, Ref, RefMut},
    },
    DashMap,
};
use tracing::trace;

#[derive(Debug, Default)]
pub struct TrashMap<Key, Value>(DashMap<Key, Value>)
where
    Key: Eq + Hash;

impl<Key, Value> TrashMap<Key, Value>
where
    Key: Eq + Hash,
{
    pub fn new() -> Self {
        TrashMap(DashMap::new())
    }

    pub fn insert(&self, key: Key, value: Value) {
        trace!("write locking");
        self.0.insert(key, value);
        trace!("write unlocking");
    }

    pub fn get(&self, key: &Key) -> Option<TracedRef<'_, Key, Value>> {
        trace!("read locking");
        TracedRef::try_new(self.0.get(key))
    }

    pub fn get_mut(&self, key: &Key) -> Option<TracedRefMut<'_, Key, Value>> {
        trace!("write locking");
        TracedRefMut::try_new(self.0.get_mut(key))
    }

    pub fn entry(&self, key: Key) -> TracedEntry<'_, Key, Value> {
        trace!("write locking");
        TracedEntry(Some(self.0.entry(key)))
    }
}

pub struct TracedRef<'entry, Key, Value>(Option<Ref<'entry, Key, Value>>);

impl<'entry, Key, Value> TracedRef<'entry, Key, Value>
where
    Key: Eq + Hash,
{
    pub fn try_new(maybe_ref: Option<Ref<'entry, Key, Value>>) -> Option<Self> {
        match maybe_ref {
            Some(inner) => Some(TracedRef(Some(inner))),
            None => {
                trace!("read unlocking");
                None
            }
        }
    }

    pub fn map<Output>(
        mut self,
        mapper: impl FnOnce(&Value) -> &Output,
    ) -> TracedMappedRef<'entry, Key, Value, Output> {
        TracedMappedRef(Some(self.0.take().unwrap().map(mapper)))
    }
}

impl<'entry, Key, Value> Deref for TracedRef<'entry, Key, Value> {
    type Target = Ref<'entry, Key, Value>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<Key, Value> Drop for TracedRef<'_, Key, Value> {
    fn drop(&mut self) {
        if self.0.is_some() {
            trace!("read unlocking");
        }
    }
}

pub struct TracedRefMut<'entry, Key, Value>(Option<RefMut<'entry, Key, Value>>);

impl<'entry, Key, Value> TracedRefMut<'entry, Key, Value>
where
    Key: Eq + Hash,
{
    pub fn try_new(maybe_ref: Option<RefMut<'entry, Key, Value>>) -> Option<Self> {
        match maybe_ref {
            Some(inner) => Some(TracedRefMut(Some(inner))),
            None => {
                trace!("write unlocking");
                None
            }
        }
    }

    pub fn map<Output>(
        mut self,
        mapper: impl FnOnce(&mut Value) -> &mut Output,
    ) -> TracedMappedRefMut<'entry, Key, Value, Output> {
        TracedMappedRefMut(Some(self.0.take().unwrap().map(mapper)))
    }
}

impl<'entry, Key, Value> Deref for TracedRefMut<'entry, Key, Value>
where
    Key: Eq + Hash,
{
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<'entry, Key, Value> DerefMut for TracedRefMut<'entry, Key, Value>
where
    Key: Eq + Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

impl<Key, Value> Drop for TracedRefMut<'_, Key, Value> {
    fn drop(&mut self) {
        if self.0.is_some() {
            trace!("write unlocking");
        }
    }
}

pub struct TracedMappedRef<'entry, Key, Value, Output>(
    Option<MappedRef<'entry, Key, Value, Output>>,
);

impl<'entry, Key, Value, Output> TracedMappedRef<'entry, Key, Value, Output>
where
    Key: Eq + Hash,
{
    pub fn map<NewOutput>(
        mut self,
        mapper: impl FnOnce(&Output) -> &NewOutput,
    ) -> TracedMappedRef<'entry, Key, Value, NewOutput> {
        TracedMappedRef(Some(self.0.take().unwrap().map(mapper)))
    }
}

impl<'entry, Key, Value, Output> Deref for TracedMappedRef<'entry, Key, Value, Output>
where
    Key: Eq + Hash,
{
    type Target = Output;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<Key, Value, Output> Drop for TracedMappedRef<'_, Key, Value, Output> {
    fn drop(&mut self) {
        if self.0.is_some() {
            trace!("read unlocking");
        }
    }
}

pub struct TracedMappedRefMut<'entry, Key, Value, Output>(
    Option<MappedRefMut<'entry, Key, Value, Output>>,
);

impl<'entry, Key, Value, Output> TracedMappedRefMut<'entry, Key, Value, Output>
where
    Key: Eq + Hash,
{
    pub fn map<NewOutput>(
        mut self,
        mapper: impl FnOnce(&mut Output) -> &mut NewOutput,
    ) -> TracedMappedRefMut<'entry, Key, Value, NewOutput> {
        TracedMappedRefMut(Some(self.0.take().unwrap().map(mapper)))
    }
}

impl<'entry, Key, Value, Output> Deref for TracedMappedRefMut<'entry, Key, Value, Output>
where
    Key: Eq + Hash,
{
    type Target = Output;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<'entry, Key, Value, Output> DerefMut for TracedMappedRefMut<'entry, Key, Value, Output>
where
    Key: Eq + Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

impl<Key, Value, Output> Drop for TracedMappedRefMut<'_, Key, Value, Output> {
    fn drop(&mut self) {
        if self.0.is_some() {
            trace!("write unlocking");
        }
    }
}

pub struct TracedEntry<'entry, Key, Value>(Option<Entry<'entry, Key, Value>>);

impl<'entry, Key, Value> TracedEntry<'entry, Key, Value>
where
    Key: Eq + Hash,
{
    pub fn insert(mut self, value: Value) -> TracedRefMut<'entry, Key, Value> {
        TracedRefMut(Some(self.0.take().unwrap().insert(value)))
    }
}

impl<Key, Value> Drop for TracedEntry<'_, Key, Value> {
    fn drop(&mut self) {
        if self.0.is_some() {
            trace!("write unlocking");
        }
    }
}
