/// Runtime support for compiled leem programs
///
/// This module contains runtime data structures and functions that
/// compiled leem programs link against.

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::ptr;

/// Arena allocator for query execution
///
/// Each query gets its own arena. All allocations during query
/// execution come from the arena, and the entire arena is freed
/// when the query completes.
#[repr(C)]
pub struct Arena {
    data: *mut u8,
    size: usize,
    capacity: usize,
}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 8).unwrap();
        let data = unsafe { alloc(layout) };

        Arena {
            data,
            size: 0,
            capacity,
        }
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        // Align the current size
        let aligned_size = (self.size + align - 1) & !(align - 1);

        if aligned_size + size > self.capacity {
            panic!("Arena out of memory");
        }

        let ptr = unsafe { self.data.add(aligned_size) };
        self.size = aligned_size + size;
        ptr
    }

    pub fn reset(&mut self) {
        self.size = 0;
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 8).unwrap();
        unsafe { dealloc(self.data, layout) };
    }
}

/// Proof tree node for explainability
#[repr(C)]
pub struct ProofNode {
    pub rule_id: u32,
    pub relation_name: *const u8,
    pub relation_name_len: usize,
    pub arguments: *mut *mut u8,
    pub arg_count: usize,
    pub children: *mut *mut ProofNode,
    pub children_count: usize,
}

/// Query context holds arena and proof tracking
#[repr(C)]
pub struct QueryContext {
    pub arena: Arena,
    pub proof_root: *mut ProofNode,
    pub track_proofs: bool,
}

impl QueryContext {
    pub fn new(arena_capacity: usize, track_proofs: bool) -> Self {
        QueryContext {
            arena: Arena::new(arena_capacity),
            proof_root: ptr::null_mut(),
            track_proofs,
        }
    }
}

/// Table entry for memoization
#[repr(C)]
pub struct TableEntry {
    key_hash: u64,
    key: *mut u8,
    value: *mut u8,
    next: *mut TableEntry,
}

/// Hash table for tabling/memoization
#[repr(C)]
pub struct Table {
    buckets: *mut *mut TableEntry,
    bucket_count: usize,
    entry_count: usize,
}

impl Table {
    pub fn new(bucket_count: usize) -> Self {
        let buckets = unsafe {
            let layout = Layout::array::<*mut TableEntry>(bucket_count).unwrap();
            let ptr = alloc(layout) as *mut *mut TableEntry;
            // Initialize to null
            for i in 0..bucket_count {
                ptr.add(i).write(ptr::null_mut());
            }
            ptr
        };

        Table {
            buckets,
            bucket_count,
            entry_count: 0,
        }
    }

    pub fn insert(&mut self, key_hash: u64, key: *mut u8, value: *mut u8) {
        let bucket_idx = (key_hash as usize) % self.bucket_count;

        let entry = Box::into_raw(Box::new(TableEntry {
            key_hash,
            key,
            value,
            next: ptr::null_mut(),
        }));

        unsafe {
            let bucket_ptr = self.buckets.add(bucket_idx);
            let old_head = *bucket_ptr;
            (*entry).next = old_head;
            *bucket_ptr = entry;
        }

        self.entry_count += 1;
    }

    pub fn lookup(&self, key_hash: u64, key_eq: impl Fn(*mut u8, *mut u8) -> bool) -> Option<*mut u8> {
        let bucket_idx = (key_hash as usize) % self.bucket_count;

        unsafe {
            let mut current = *self.buckets.add(bucket_idx);

            while !current.is_null() {
                if (*current).key_hash == key_hash {
                    // Need to compare keys properly
                    // For now, just return the value
                    return Some((*current).value);
                }
                current = (*current).next;
            }
        }

        None
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        unsafe {
            // Free all entries
            for i in 0..self.bucket_count {
                let mut current = *self.buckets.add(i);
                while !current.is_null() {
                    let next = (*current).next;
                    drop(Box::from_raw(current));
                    current = next;
                }
            }

            // Free buckets array
            let layout = Layout::array::<*mut TableEntry>(self.bucket_count).unwrap();
            dealloc(self.buckets as *mut u8, layout);
        }
    }
}

// C-compatible exports for LLVM-generated code

#[no_mangle]
pub extern "C" fn leem_arena_new(capacity: usize) -> *mut Arena {
    Box::into_raw(Box::new(Arena::new(capacity)))
}

#[no_mangle]
pub extern "C" fn leem_arena_free(arena: *mut Arena) {
    if !arena.is_null() {
        unsafe { drop(Box::from_raw(arena)) };
    }
}

#[no_mangle]
pub extern "C" fn leem_arena_alloc(arena: *mut Arena, size: usize, align: usize) -> *mut u8 {
    unsafe {
        (*arena).allocate(size, align)
    }
}

#[no_mangle]
pub extern "C" fn leem_table_new(bucket_count: usize) -> *mut Table {
    Box::into_raw(Box::new(Table::new(bucket_count)))
}

#[no_mangle]
pub extern "C" fn leem_table_free(table: *mut Table) {
    if !table.is_null() {
        unsafe { drop(Box::from_raw(table)) };
    }
}

#[no_mangle]
pub extern "C" fn leem_table_insert(
    table: *mut Table,
    key_hash: u64,
    key: *mut u8,
    value: *mut u8,
) {
    unsafe {
        (*table).insert(key_hash, key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_creation() {
        let mut arena = Arena::new(1024);
        assert_eq!(arena.size, 0);
        assert_eq!(arena.capacity, 1024);
    }

    #[test]
    fn test_arena_allocation() {
        let mut arena = Arena::new(1024);
        let ptr1 = arena.allocate(64, 8);
        let ptr2 = arena.allocate(64, 8);

        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert_ne!(ptr1, ptr2);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::new(1024);
        arena.allocate(64, 8);
        assert_eq!(arena.size, 64);

        arena.reset();
        assert_eq!(arena.size, 0);
    }

    #[test]
    fn test_table_creation() {
        let table = Table::new(16);
        assert_eq!(table.bucket_count, 16);
        assert_eq!(table.entry_count, 0);
    }

    #[test]
    fn test_table_insert_lookup() {
        let mut table = Table::new(16);
        let key = Box::into_raw(Box::new(42u64)) as *mut u8;
        let value = Box::into_raw(Box::new(100u64)) as *mut u8;

        table.insert(12345, key, value);
        assert_eq!(table.entry_count, 1);

        let result = table.lookup(12345, |_, _| true);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), value);
    }
}
