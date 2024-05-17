use fixedbitset::FixedBitSet;

use crate::Publish;

/// `OutgoingPublishBucket` is a struct that represents a bucket of outgoing publish messages.
/// It contains a vector of `Option<Publish>` messages.
#[derive(Debug, Clone)]
pub struct OutgoingPublishBucket {
    vec: Vec<Option<Publish>>,
}

/// `OutOfBounds` is a struct that represents an out of bounds error.
/// It contains a `u16` value that represents the packet identifier that caused the error.
#[derive(Debug, Copy, Clone)]
pub struct OutOfBounds(pub u16);

impl OutgoingPublishBucket {
    /// Creates a new `OutgoingPublishBucket` with a specified limit.
    /// The limit is used to initialize the vector with `None` values.
    pub fn with_limit(max_pkid: u16) -> Self {
        Self {
            vec: vec![None; max_pkid as usize + 1],
        }
    }

    /// Returns the number of `Some` values in the vector.
    pub fn len(&self) -> usize {
        self.vec.iter().filter(|x| x.is_some()).count()
    }

    /// Inserts a `Publish` value into the vector at the position specified by its packet identifier.
    /// If the position is out of bounds, it returns an `OutOfBounds` error.
    pub fn insert(&mut self, value: Publish) -> Result<Option<Publish>, OutOfBounds> {
        let index = value.pkid as usize;
        match self.vec.get_mut(index) {
            Some(slot) => Ok(slot.replace(value)),
            None => Err(OutOfBounds(value.pkid)),
        }
    }

    /// Removes a `Publish` value from the vector at the specified position.
    /// If the position is out of bounds, it returns an `OutOfBounds` error.
    pub fn remove(&mut self, pkid: u16) -> Result<Option<Publish>, OutOfBounds> {
        let index = pkid as usize;
        self.vec.get_mut(index)
            .map(|slot| Ok(slot.take()))
            .unwrap_or_else(|| Err(OutOfBounds(pkid)))
    }

    /// Returns a reference to the `Publish` value at the specified position.
    /// If the position is out of bounds, it returns an `OutOfBounds` error.
    pub fn get(&self, pkid: u16) -> Result<Option<&Publish>, OutOfBounds> {
        self.vec.get(pkid as usize)
            .map(|opt| Ok(opt.as_ref()))
            .unwrap_or_else(|| Err(OutOfBounds(pkid)))
    }

    /// Drains the `Publish` values into another vector, transforming them using a specified function.
    /// The transformation starts from the position specified by `last_puback`.
   pub fn drain_into<T>(&mut self, vec: &mut Vec<T>, last_puback: usize, map: fn(Publish) -> T) {
        for (index, opt_publish) in self.vec.iter_mut().enumerate() {
            if let Some(publish) = opt_publish.take() {
                if index >= last_puback {
                    vec.push(map(publish));
                }
            }
        }
        self.vec.clear();
    }
}

impl From<Vec<Option<Publish>>> for OutgoingPublishBucket {
    /// Converts a vector of `Option<Publish>` values into an `OutgoingPublishBucket`.
    fn from(vec: Vec<Option<Publish>>) -> Self {
        Self { vec }
    }
}

/// `PkidSet` is a struct that represents a set of packet identifiers (pkids).
/// It uses a `FixedBitSet` to efficiently store the pkids.
#[derive(Debug, Clone)]
pub struct PkidSet {
    set: FixedBitSet,
}

impl PkidSet {
    /// Creates a new `PkidSet` with a specified limit.
    /// The limit is used to initialize the `FixedBitSet` with a certain capacity.
    pub fn with_limit(max_pkid: u16) -> Self {
        Self {
            set: FixedBitSet::with_capacity(max_pkid as usize + 1),
        }
    }

    /// Creates a new `PkidSet` that can store the full range of `u16` values.
    pub fn full_range() -> Self {
        Self::with_limit(u16::MAX)
    }

    /// Returns the number of ones (i.e., set bits) in the `FixedBitSet`.
    pub fn len(&self) -> usize {
        self.set.count_ones(..)
    }

    /// Inserts a pkid into the `FixedBitSet`.
    /// If the pkid is out of bounds, it returns an `OutOfBounds` error.
    pub fn insert(&mut self, pkid: u16) -> Result<bool, OutOfBounds> {
        let index = pkid as usize;
        if index >= self.set.len() {
            return Err(OutOfBounds(pkid));
        }
        let was_present = self.set.put(index);
        Ok(was_present)
    }

    /// Checks if a pkid is in the `FixedBitSet`.
    /// This method is only available in test configurations.
    #[cfg(test)]
    pub fn contains(&self, pkid: u16) -> bool {
        self.set.contains(pkid as usize)
    }

    /// Removes a pkid from the `FixedBitSet`.
    /// If the pkid is out of bounds, it returns an `OutOfBounds` error.
    pub fn remove(&mut self, pkid: u16) -> Result<bool, OutOfBounds> {
        let index = pkid as usize;
        if index >= self.set.len() {
            return Err(OutOfBounds(pkid));
        }
        let was_present = self.set.contains(index);
        self.set.set(index, false);
        Ok(was_present)
    }

    /// Drains the pkids into another vector, transforming them using a specified function.
    pub fn drain_into<T>(&mut self, vec: &mut Vec<T>, map: impl Fn(u16) -> T) {
        for pkid in self.set.ones() {
            vec.push(map(pkid as u16));
        }
        self.clear();
    }

    /// Clears the `FixedBitSet`, removing all pkids.
    pub fn clear(&mut self) {
        self.set.clear();
    }
}
