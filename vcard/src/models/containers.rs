use std::{collections::HashMap, fmt::Display};

use crate::{Alternative, Preferable, errors::VCardError};

pub struct MultiAltIDContainer<T: Alternative>(HashMap<String, AltIDContainer<T>>);

impl<T: Alternative> Default for MultiAltIDContainer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Alternative + Display> Display for MultiAltIDContainer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for val in self.0.values() {
            val.fmt(f)?;
        }
        Ok(())
    }
}

impl<T: Alternative> MultiAltIDContainer<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_value(&mut self, value: T) -> Result<(), VCardError> {
        if self.0.contains_key(value.get_alt_id()) {
            let container = self.0.get_mut(value.get_alt_id()).unwrap();
            container.add_value(value)?;
        } else {
            let altid = value.get_alt_id().to_string();
            let container = AltIDContainer::from_vec(vec![value]);
            self.0.insert(altid, container);
        }

        Ok(())
    }

    pub fn values(&self) -> &HashMap<String, AltIDContainer<T>> {
        &self.0
    }

    pub fn take_values(self) -> HashMap<String, AltIDContainer<T>> {
        self.0
    }
}

impl<T: Alternative + Preferable> MultiAltIDContainer<T> {
    /// returns the prefered value.
    ///
    /// Preference values are ascending. No guarantees are made when multiple values have the same `pref`
    pub fn get_prefered_value(&self) -> Option<&T> {
        let mut prefered_item = None;
        for (_key, container) in self.0.iter() {
            let container_prefered_item = if let Some(cpi) = container.get_prefered_value() {
                cpi
            } else {
                continue;
            };
            if prefered_item.is_none() {
                prefered_item = Some(container_prefered_item);
            } else if prefered_item.unwrap().get_alt_id() > container_prefered_item.get_alt_id() {
                prefered_item = Some(container_prefered_item);
            }
        }

        prefered_item
    }
}

/// In vcard, if multiple entries share the same type and altid, they are considered
/// to be one record. This means, all entries in an `AltIDContainer` are considered one record as well.
#[derive(Default)]
pub struct AltIDContainer<T: Alternative>(Vec<T>);

impl<T> Display for AltIDContainer<T>
where
    T: Alternative + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.0.iter() {
            item.fmt(f)?;
        }
        Ok(())
    }
}

impl<T: Alternative> AltIDContainer<T> {
    pub fn new() -> Self {
        AltIDContainer(Vec::new())
    }

    pub fn from_vec(items: Vec<T>) -> Self {
        AltIDContainer(items)
    }

    /// Adds a new value to this container.
    ///
    /// This will fail if `item` has a different `altid` than previous elements of this container.
    /// In case the container does not have any elemts, it will simply be added to the collection.
    pub fn add_value(&mut self, item: T) -> Result<(), VCardError> {
        if self.0.len() == 0 {
            self.0.push(item);
            return Ok(());
        }
        let prev_altid = self.0.get(0).unwrap().get_alt_id();
        if prev_altid != item.get_alt_id() {
            return Err(VCardError::InvalidAltID {
                expected_altid: prev_altid.to_string(),
                actual_altid: item.get_alt_id().to_owned(),
            });
        }

        self.0.push(item);

        Ok(())
    }

    pub fn values(&self) -> &[T] {
        &self.0
    }

    pub fn take_values(self) -> Vec<T> {
        self.0
    }
}

impl<T> AltIDContainer<T>
where
    T: Alternative + Preferable,
{
    /// returns the prefered value.
    ///
    /// Preference values are ascending. No guarantees are made when multiple values have the same `pref`
    pub fn get_prefered_value(&self) -> Option<&T> {
        let mut prefered_item = None;
        for item in self.0.iter() {
            if prefered_item.is_none() {
                prefered_item = Some(item);
                continue;
            }

            if prefered_item.unwrap().get_alt_id() > item.get_alt_id() {
                prefered_item = Some(item);
            }
        }

        prefered_item
    }
}
