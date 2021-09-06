use generational_arena::{
    Arena, Drain as ArenaDrain, Index as ArenaIndex, Iter as ArenaIter, IterMut as ArenaIterMut,
};
use std::marker::PhantomData;
pub enum RemoveStatus<T> {
    Removed(T),
    DoesNotExist,
}
pub struct AssetIter<'a, T> {
    iter: ArenaIter<'a, T>,
}
impl<'a, T> Iterator for AssetIter<'a, T> {
    type Item = (AssetHandle<T>, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((index, data)) => Some((
                AssetHandle {
                    index,
                    asset_type: PhantomData,
                },
                data,
            )),
            None => None,
        }
    }
}
pub struct AssetIterMut<'a, T> {
    iter: ArenaIterMut<'a, T>,
}
impl<'a, T> Iterator for AssetIterMut<'a, T> {
    type Item = (AssetHandle<T>, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((index, data)) => Some((
                AssetHandle {
                    index,
                    asset_type: PhantomData,
                },
                data,
            )),
            None => None,
        }
    }
}
pub struct AssetDrain<'a, T> {
    drain: ArenaDrain<'a, T>,
}
impl<'a, T> Iterator for AssetDrain<'a, T> {
    type Item = (AssetHandle<T>, T);
    fn next(&mut self) -> Option<Self::Item> {
        match self.drain.next() {
            Some((index, data)) => Some((
                AssetHandle {
                    index,
                    asset_type: PhantomData,
                },
                data,
            )),
            None => None,
        }
    }
}
/// Manages asset in a sukakpak based game.
pub struct AssetManager<T> {
    arena: Arena<T>,
}
impl<T> AssetManager<T> {
    /// Inserts an asset into the manager
    /// ```
    /// # use asset_manager::AssetManager;
    /// let mut manager: AssetManager<u8> = Default::default();
    /// manager.insert(1);
    /// ```
    pub fn insert(&mut self, data: T) -> AssetHandle<T> {
        AssetHandle {
            index: self.arena.insert(data),
            asset_type: PhantomData,
        }
    }
    /// Retrives data from the manager
    /// ```
    /// # use asset_manager::AssetManager;
    /// let mut manager: AssetManager<u8> = Default::default();
    /// let one = manager.insert(1);
    /// assert_eq!(*manager.get(&one).unwrap(),1);
    /// ```
    pub fn get(&self, handle: &AssetHandle<T>) -> Option<&T> {
        self.arena.get(handle.index)
    }
    /// Gets mutable refrence to an element
    /// ```
    /// # use asset_manager::AssetManager;
    /// let mut manager: AssetManager<u8> = Default::default();
    /// let one = manager.insert(1);
    /// *manager.get_mut(&one).unwrap()+=1;
    /// assert_eq!(*manager.get(&one).unwrap(),2);
    /// ```
    pub fn get_mut(&mut self, handle: &AssetHandle<T>) -> Option<&mut T> {
        self.arena.get_mut(handle.index)
    }
    /// Removes element. Returns element that was removed
    /// ```
    /// # use asset_manager::{AssetManager,RemoveStatus};
    /// let mut manager: AssetManager<u8> = Default::default();
    /// let one = manager.insert(1);
    /// match manager.remove(one){
    ///     RemoveStatus::Removed(o)=>assert_eq!(o,1),
    ///     RemoveStatus::DoesNotExist=>panic!("invalid state")
    /// }
    /// ```
    pub fn remove(&mut self, handle: AssetHandle<T>) -> RemoveStatus<T> {
        match self.arena.remove(handle.index) {
            Some(data) => RemoveStatus::Removed(data),
            None => RemoveStatus::DoesNotExist,
        }
    }
    /// Removes all elements from the asset manager
    /// ```
    /// # use asset_manager::AssetManager;
    /// let mut manager: AssetManager<u8> = Default::default();
    /// manager.insert(1);
    /// assert_eq!(manager.drain().map(|(_i,data)|data).collect::<Vec<_>>(),vec![1]);
    ///```
    pub fn drain(&mut self) -> AssetDrain<'_, T> {
        AssetDrain {
            drain: self.arena.drain(),
        }
    }
    /// Iterates through all elements. Order of iteration not defined.
    /// ```
    /// # use asset_manager::AssetManager;
    /// let mut manager: AssetManager<u8> = Default::default();
    /// manager.insert(1);
    /// assert_eq!(manager.iter().map(|(_idx,data)|*data).collect::<Vec<_>>(),vec![1]);
    ///```
    pub fn iter(&self) -> AssetIter<'_, T> {
        AssetIter {
            iter: self.arena.iter(),
        }
    }
    /// ```
    /// # use asset_manager::AssetManager;
    /// let mut manager: AssetManager<u8> = Default::default();
    /// manager.insert(1);
    /// manager.insert(2);
    /// for i in manager.iter_mut(){
    ///     
    ///     *(i.1)+=1;
    /// }
    /// assert_eq!(manager.iter().map(|(_i,n)|*n).collect::<Vec<_>>(),vec![2,3]);
    ///```
    pub fn iter_mut(&mut self) -> AssetIterMut<'_, T> {
        AssetIterMut {
            iter: self.arena.iter_mut(),
        }
    }
}
impl<T> Default for AssetManager<T> {
    fn default() -> Self {
        Self {
            arena: Default::default(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct AssetHandle<T> {
    index: ArenaIndex,
    asset_type: PhantomData<T>,
}
impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            asset_type: PhantomData,
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
