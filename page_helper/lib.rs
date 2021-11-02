#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use ink_lang as ink;
pub use self::page_helper::PageParams;
pub use self::page_helper::PageResult;
pub use self::page_helper::cal_pages;

#[ink::contract]
mod page_helper {

    use alloc::string::String;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        traits::{
            PackedLayout,
            SpreadLayout,
        },
        collections::{
            HashMap as StorageHashMap,
        }
    };

    #[derive(Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout,)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct PageParams {
        pub page_index: u64,
        pub page_size: u64,
    }

    #[derive(Debug, scale::Encode, scale::Decode, Clone, )]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, )
    )]
    pub struct PageResult<T> {
        pub success: bool,
        pub err: String,
        pub total: u64,
        pub pages: u64,
        pub page_index: u64,
        pub page_size: u64,
        pub data: Vec<T>,
    }

    pub fn cal_pages(params: &PageParams, total: u64) -> (u64, u64, u64) {
        let start = params.page_index * params.page_size;
        let mut end = start + params.page_size;
        if end > total {
            end = total
        }
        assert!(params.page_size <= 0 || start >= total || start < end, "wrong params");
        let mut pages = total / params.page_size;
        if total % params.page_size > 0 {
            pages += 1;
        }
        (start, end, pages)
    }

    #[ink(storage)]
    pub struct page_helper {
        dummy: u64,
    }

    impl page_helper {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                dummy: 0
            }
        }

        /// A message that init a service.
        #[ink(message)]
        pub fn query(&self) -> u64 {
            self.dummy
        }
    }
    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        fn test_query_page(params: PageParams) -> PageResult<u64> {
            let mut data = Vec::new();
            data.push(1);
            data.push(2);
            return PageResult{
                success: true,
                err: "success",
                total: 100,
                pages: 1,
                page_index: params.page_index,
                page_size: params.page_size,
                data,
            }
        }
        /// We test if the default constructor does its job.
        #[ink::test]
        fn page_works() {
            let params = PageParams { page_index: 1, page_size: 1 };
            let res = test_query_page(params);
            assert_eq!(2, res.data.len());
        }
    }
}
