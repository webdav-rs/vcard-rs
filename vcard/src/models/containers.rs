use std::{collections::HashMap, fmt::Display};

use crate::{errors::VCardError, Alternative, Preferable};

#[derive(PartialEq, Debug)]
pub struct MultiAltIDContainer<T: Alternative + PartialEq + std::fmt::Debug>(
    HashMap<String, AltIDContainer<T>>,
);

impl<T: Alternative + PartialEq + std::fmt::Debug> Default for MultiAltIDContainer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Alternative + Display + PartialEq + std::fmt::Debug> Display for MultiAltIDContainer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for val in self.0.values() {
            val.fmt(f)?;
        }
        Ok(())
    }
}

impl<T: Alternative + PartialEq + std::fmt::Debug> MultiAltIDContainer<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_value(&mut self, value: T) {
        if self.0.contains_key(value.get_alt_id()) {
            let container = self.0.get_mut(value.get_alt_id()).unwrap();
            container
                .add_value(value)
                .expect("we have checked the key beforehand. What is this trickery!?");
        } else {
            let altid = value.get_alt_id().to_string();
            let container = AltIDContainer::from_vec(vec![value]);
            self.0.insert(altid, container);
        }
    }

    pub fn values(&self) -> &HashMap<String, AltIDContainer<T>> {
        &self.0
    }

    pub fn take_values(self) -> HashMap<String, AltIDContainer<T>> {
        self.0
    }
}

impl<T: Alternative + Preferable + PartialEq + std::fmt::Debug> MultiAltIDContainer<T> {
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
            } else if prefered_item.unwrap().get_pref() > container_prefered_item.get_pref() {
                prefered_item = Some(container_prefered_item);
            }
        }

        prefered_item
    }
}

/// In vcard, if multiple entries share the same type and altid, they are considered
/// to be one record. This means, all entries in an `AltIDContainer` are considered one record as well.
#[derive(Default, PartialEq,Debug)]
pub struct AltIDContainer<T: Alternative + std::fmt::Debug>(Vec<T>);

impl<T> Display for AltIDContainer<T>
where
    T: Alternative + Display + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.0.iter() {
            Display::fmt(&item, f)?;
        }
        Ok(())
    }
}

impl<T: Alternative + std::fmt::Debug> AltIDContainer<T> {
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
    T: Alternative + Preferable + std::fmt::Debug,
{
    /// returns the prefered value.
    ///
    /// Preference values are ascending. No guarantees are made when multiple values have the same `pref`
    pub fn get_prefered_value(&self) -> Option<&T> {
        let mut prefered_item = None;
        for item in self.0.iter() {
            if prefered_item.is_none() {
                prefered_item = Some(item);
            } else if prefered_item.unwrap().get_pref() > item.get_pref() {
                prefered_item = Some(item);
            }
        }
        prefered_item
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;
    use crate::*;

    #[test]
    fn test_altid_container() -> Result<(), Box<dyn Error>> {
        let mut container = AltIDContainer::new();
        container.add_value(FN {
            altid: Some("1".into()),
            value: "foo".into(),
            pref: Some(50),
            ..Default::default()
        })?;

        container.add_value(FN {
            altid: Some("1".into()),
            value: "bar".into(),
            pref: Some(49),
            ..Default::default()
        })?;

        // it should not be possible to input different alt ids into the same container
        let result = container.add_value(FN {
            altid: Some("2".into()),
            value: "foobar".into(),
            ..Default::default()
        });
        assert!(result.is_err());

        let prefered_val = container.get_prefered_value().expect("expect a value here");
        assert_eq!(prefered_val.value, "bar".to_string());

        Ok(())
    }

    #[test]
    fn test_multi_altid_container() -> Result<(), Box<dyn Error>> {
        let mut testant = MultiAltIDContainer::default();
        testant.add_value(FN {
            altid: Some("1".into()),
            value: "foo".into(),
            ..Default::default()
        });
        testant.add_value(FN {
            altid: Some("1".into()),
            value: "bar".into(),
            pref: Some(49),
            ..Default::default()
        });
        testant.add_value(FN {
            altid: Some("2".into()),
            value: "foobar".into(),
            pref: Some(1),
            ..Default::default()
        });
        let pref = testant
            .get_prefered_value()
            .expect("expect a prefered value here");
        assert_eq!(pref.value, "foobar".to_string());
        Ok(())
    }
}
