#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use ink_lang as ink;

#[ink::contract]
mod polkasion {
    use alloc::string::String;
    use ink_prelude::vec::Vec;
    use ink_prelude::collections::BTreeMap;
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        traits::{PackedLayout, SpreadLayout},
    };

    use page_helper::{PageParams, PageResult, cal_pages};

    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct StorageInfo {
        hash: Hash,
        creator: AccountId,
        // for what, like document comment
        usage: String,
        // save in what storage, like ipfs
        save_at: String,
        // resource address
        url: String,
    }

    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct SignInfo {
        addr: AccountId,
        create_at: u64,
    }

    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct AgreementInfo {
        index: u64,
        creator: AccountId,
        name: String,
        create_at: u64,
        // init=0, waiting=1, finished=2
        status: u8,
        signers: Vec<AccountId>,
        agreement_file: StorageInfo,
        // map signs: accountId -> sign
        signs: BTreeMap<AccountId, [u8; 64]>,
        sign_infos: BTreeMap<AccountId, SignInfo>,
        // map resources: accountId -> resources vec
        // like comment
        resources: BTreeMap<AccountId, Vec<StorageInfo>>,
    }

    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct CreateAgreementParams {
        name: String,
        signers: Vec<AccountId>,
        agreement_file: StorageInfo,
    }

    #[ink(event)]
    pub struct CreateAgreementEvent {
        index: u64,
        creator: AccountId,
        name: String,
    }

    #[ink(event)]
    pub struct AttachAgreementEvent {
        index: u64,
        hash: Hash,
        creator: AccountId,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Polkasign {
        owner: AccountId,
        index: u64,
        agreements_map: StorageHashMap<u64, AgreementInfo>,
        agreements_creator_map: StorageHashMap<AccountId, Vec<u64>>,
        agreements_collaborator_map: StorageHashMap<AccountId, Vec<u64>>,
    }

    impl Polkasign {
        #[ink(constructor)]
        pub fn new(owner: AccountId) -> Self {
            Self {
                owner,
                index: 0,
                agreements_map: StorageHashMap::new(),
                agreements_creator_map: StorageHashMap::new(),
                agreements_collaborator_map: StorageHashMap::new()
            }
        }

        #[ink(message)]
        pub fn create_agreement(&mut self, params: CreateAgreementParams) -> u64 {
            let caller = self.env().caller();
            assert!(self.index + 1 > self.index, "index overflow");
            let index = self.index;
            self.index += 1;

            // save in contract
            let creator_ids = self.agreements_creator_map.entry(caller.clone()).or_insert(Vec::new());
            creator_ids.push(index);
            for i in params.signers.iter() {
                let tmp_ids = self.agreements_collaborator_map.entry(i.clone()).or_insert(Vec::new());
                tmp_ids.push(index);
            }

            let mut storage_info = params.agreement_file;
            storage_info.creator = caller;
            let info = AgreementInfo{
                index,
                creator: caller,
                name: params.name.clone(),
                create_at: self.env().block_timestamp(),
                status: 0,
                signers: params.signers,
                agreement_file: storage_info,
                signs: BTreeMap::new(),
                sign_infos: BTreeMap::new(),
                resources: BTreeMap::new(),
            };
            self.agreements_map.insert(index, info);
            self.env().emit_event(CreateAgreementEvent {
                index,
                creator: caller,
                name: params.name,
            });
            index
        }

        #[ink(message)]
        pub fn create_agreement_with_sign(&mut self, params: CreateAgreementParams, sign: [u8; 64]) {
            let caller = self.env().caller();
            let time_at = self.env().block_timestamp();
            let index = self.create_agreement(params);
            let a = self.agreements_map.get_mut(&index).unwrap();

            let public_key = match ed25519_compact::PublicKey::from_slice(caller.as_ref()) {
                Ok(pk) => pk,
                Err(_) => panic!("covert PublicKey err"),
            };

            let sig = match ed25519_compact::Signature::from_slice(&sign[..]) {
                Ok(s) => s,
                Err(_) => panic!("covert Signature err"),
            };

            assert!(public_key.verify(a.agreement_file.hash, &sig).is_ok(), "Signature wrong");

            // if sign enough, set waiting
            a.status = 1;
            a.signs.insert(caller, sign);
            a.sign_infos.insert(caller, SignInfo{
                addr: caller,
                create_at: time_at,
            });
        }

        #[ink(message)]
        pub fn attach_resource_to_agreement(&mut self, index: u64, info: StorageInfo) {
            let caller = self.env().caller();
            let agreement = self.agreements_map.get_mut(&index).unwrap();
            assert!(agreement.signers.contains(&caller), "not found in signers");

            let storage_hash = info.hash;
            let resources = agreement.resources.entry(caller.clone()).or_insert(Vec::new());
            resources.push(info);
            self.env().emit_event(AttachAgreementEvent {
                index,
                hash: storage_hash,
                creator: caller,
            });
        }

        #[ink(message)]
        pub fn attach_resource_with_sign(&mut self, index: u64, info: StorageInfo, sign: [u8; 64]) {
            let caller = self.env().caller();
            let time_at = self.env().block_timestamp();
            self.attach_resource_to_agreement(index, info);
            let agreement = self.agreements_map.get_mut(&index).unwrap();

            let public_key = match ed25519_compact::PublicKey::from_slice(caller.as_ref()) {
                Ok(pk) => pk,
                Err(_) => panic!("covert PublicKey err"),
            };

            let sig = match ed25519_compact::Signature::from_slice(&sign[..]) {
                Ok(s) => s,
                Err(_) => panic!("covert Signature err"),
            };

            assert!(public_key.verify(agreement.agreement_file.hash, &sig).is_ok(), "Signature wrong");

            agreement.signs.insert(caller, sign);
            agreement.sign_infos.insert(caller, SignInfo{
                addr: caller,
                create_at: time_at,
            });

            // if sign enough, set finished
            if agreement.signs.len() >= agreement.signers.len() {
                agreement.status = 2;
            }
        }

        #[ink(message)]
        pub fn check_sign(&mut self, msg: [u8; 32], sign: [u8; 64]) -> bool {

            let caller = self.env().caller();
            let public_key = match ed25519_compact::PublicKey::from_slice(caller.as_ref()) {
                Ok(pk) => pk,
                Err(_) => panic!("covert PublicKey err"),
            };

            let sig = match ed25519_compact::Signature::from_slice(&sign[..]) {
                Ok(s) => s,
                Err(_) => panic!("covert Signature err"),
            };

            public_key.verify(msg, &sig).is_ok()
        }

        #[ink(message)]
        pub fn query_agreement_by_creator(&mut self, creator: AccountId, pageParams: PageParams) -> PageResult<AgreementInfo> {
            let list_res = self.agreements_creator_map.get(&creator);
            if list_res.is_none() {
                return PageResult{
                    success: true,
                    err: String::from("success"),
                    total: 0,
                    pages: 0,
                    page_index: 0,
                    page_size: 0,
                    data: Vec::new(),
                }
            }
            let ids = list_res.unwrap();
            let total = ids.len() as u64;
            let (start, end, pages) = cal_pages(&pageParams, total);
            let mut result = Vec::new();
            for i in start..end {
                result.push(self.agreements_map.get(&ids[i as usize]).unwrap().clone());
            }
            return PageResult{
                success: true,
                err: String::from("success"),
                total,
                pages,
                page_index: pageParams.page_index,
                page_size: pageParams.page_size,
                data: result,
            }
        }

        #[ink(message)]
        pub fn query_agreement_by_id(&mut self, index: u64) -> AgreementInfo {
            self.agreements_map.get(&index).unwrap().clone()
        }

        #[ink(message)]
        pub fn query_agreement_by_collaborator(&mut self, collaborator: AccountId, pageParams: PageParams) -> PageResult<AgreementInfo> {
            let list_res = self.agreements_collaborator_map.get(&collaborator);
            if list_res.is_none() {
                return PageResult{
                    success: true,
                    err: String::from("success"),
                    total: 0,
                    pages: 0,
                    page_index: 0,
                    page_size: 0,
                    data: Vec::new(),
                }
            }
            let ids = list_res.unwrap();
            let total = ids.len() as u64;
            let (start, end, pages) = cal_pages(&pageParams, total);
            let mut result = Vec::new();
            for i in start..end {
                result.push(self.agreements_map.get(&ids[i as usize]).unwrap().clone());
            }
            return PageResult{
                success: true,
                err: String::from("success"),
                total,
                pages,
                page_index: pageParams.page_index,
                page_size: pageParams.page_size,
                data: result,
            }
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

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let polkasion = Polkasign::default();
            assert_eq!(polkasion.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut polkasion = Polkasign::new(false);
            assert_eq!(polkasion.get(), false);
            polkasion.flip();
            assert_eq!(polkasion.get(), true);
        }
    }
}
