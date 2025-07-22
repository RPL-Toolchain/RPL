//@ ignore-on-host

use std::cmp::Eq;
use std::hash::Hash;
use std::mem;

pub struct Elem<K, V>
where
    K: 'static + Eq + Hash + Sized,
    V: 'static + Sized,
{
    key: K,
    value: V,
    hash: u64,
}

pub struct OccupiedEntry<'a, K: 'static, V: 'static + 'a>
where
    K: 'static + Eq + Hash + Sized,
    V: 'static + Sized,
{
    elem: &'a mut Elem<K, V>,
}

impl<'a, K: 'static, V: 'static + 'a> OccupiedEntry<'a, K, V>
where
    K: 'static + Eq + Hash + Sized,
    V: 'static + Sized,
{
    pub fn remove_entry(self) -> (K, V) {
        let mut key: K = unsafe { mem::uninitialized() };  //here 
        let mut value: V = unsafe { mem::uninitialized() };
        self.elem.hash |= 0x8000000000000000u64;
        mem::swap(&mut key, &mut self.elem.key);
        mem::swap(&mut value, &mut self.elem.value);
        (key, value)
    }
}