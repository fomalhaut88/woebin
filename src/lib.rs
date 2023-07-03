#![feature(test)]
extern crate test;

pub mod hierarchical_clusterizer;
pub mod woe_binning_proc;
pub mod woe_binning_proc_wrapper;

pub use crate::hierarchical_clusterizer::*;
pub use crate::woe_binning_proc::*;


pub fn binary_search<T: PartialOrd>(size: usize, arr: &[T], val: T) -> usize {
    let mut idx = 0;
    let mut size_mut = size;
    while size_mut > 1 {
        let idx_mid = idx + (size_mut >> 1);
        if arr[idx_mid] < val {
            idx = idx_mid;
        }
        size_mut = (size_mut >> 1) + (size_mut & 1);
    }
    if arr[idx] < val { idx + 1 } else { idx }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search() {
        let arr = [2, 3, 5, 7, 10, 12, 34, 123, 236, 1245];
        assert_eq!(binary_search(arr.len(), &arr, 5), 2);
        assert_eq!(binary_search(arr.len(), &arr, 2), 0);
        assert_eq!(binary_search(arr.len(), &arr, 1245), 9);
        assert_eq!(binary_search(arr.len(), &arr, 6), 3);
        assert_eq!(binary_search(arr.len(), &arr, 0), 0);
        assert_eq!(binary_search(arr.len(), &arr, 1246), 10);
    }
}
