#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use ink_lang as ink;
use ink_env::{Environment};
use ink_prelude::vec::Vec;

/// Define the operations to interact with the substrate runtime
#[ink::chain_extension]
pub trait CryptoExtension {
    type ErrorCode = CryptoExtensionErr;

    #[ink(extension = 1101, returns_result = false)]
    fn fetch_random() -> [u8; 32];

    #[ink(extension = 1102, returns_result = false)]
    fn verify_sr25519(account: [u8; 32], msg: [u8; 32], sign: [u8; 64]);

    #[ink(extension = 1103, returns_result = false)]
    fn verify_sr25519_bytes(account: [u8; 32], msg: [u8; 47], sign: [u8; 64]);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CryptoExtensionErr {
    VerifyErr,
}

impl ink_env::chain_extension::FromStatusCode for CryptoExtensionErr {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::VerifyErr),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;
    type RentFraction = <ink_env::DefaultEnvironment as Environment>::RentFraction;

    type ChainExtension = CryptoExtension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod polkasign {
    use alloc::string::String;
    use ink_prelude::vec::Vec;
    use ink_prelude::collections::BTreeMap;
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        traits::{PackedLayout, SpreadLayout},
    };
    use crate::CryptoExtensionErr;

    use page_helper::{PageParams, PageResult, cal_pages};

    #[derive(Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
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

    #[derive(Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct SignInfo {
        sign: Vec<u8>,
        addr: AccountId,
        create_at: u64,
    }

    #[derive(Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
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
        sign_infos: BTreeMap<AccountId, SignInfo>,
        // map resources: accountId -> resources vec
        // like comment
        resources: BTreeMap<AccountId, Vec<StorageInfo>>,
    }

    #[derive(Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct AgreementInfoDisplay {
        index: u64,
        creator: AccountId,
        name: String,
        create_at: u64,
        // init=0, waiting=1, finished=2
        status: u8,
        signers: Vec<AccountId>,
        agreement_file: StorageInfo,
        // map signs: accountId -> sign
        sign_infos: Vec<SignInfo>,
        // map resources: accountId -> resources vec
        // like comment
        resources: Vec<StorageInfo>,
    }

    #[derive(Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
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
    pub struct UpdateAgreementEvent {
        index: u64,
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
            let a = self.agreements_map.get(&index).unwrap();

            let file_hash = a.agreement_file.hash.clone();
            assert!(self._check_sr25519_bytes_sign(*caller.as_ref(), *file_hash.as_ref(), sign.clone()), "wrong sign");
            // if sign enough, set waiting
            let a = self.agreements_map.get_mut(&index).unwrap();
            a.status = 1;
            a.sign_infos.insert(caller, SignInfo{
                sign: sign.to_vec(),
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
            self.env().emit_event(UpdateAgreementEvent {
                index,
                creator: caller,
            });
        }

        #[ink(message)]
        pub fn attach_resource_with_sign(&mut self, index: u64, info: StorageInfo, sign: [u8; 64]) {
            let caller = self.env().caller();
            let time_at = self.env().block_timestamp();
            self.attach_resource_to_agreement(index, info);
            let agreement = self.agreements_map.get(&index).unwrap();
            assert!(self._check_sr25519_bytes_sign(*caller.as_ref(), *agreement.agreement_file.hash.as_ref(), sign.clone()), "wrong sign");

            let agreement = self.agreements_map.get_mut(&index).unwrap();
            agreement.sign_infos.insert(caller, SignInfo{
                sign: sign.to_vec(),
                addr: caller,
                create_at: time_at,
            });
            agreement.status = 1;

            // if sign enough, set finished
            if agreement.sign_infos.len() >= agreement.signers.len() {
                agreement.status = 2;
            }
        }

        #[ink(message)]
        pub fn check_sr25519_sign(&mut self, msg: [u8; 32], sign: [u8; 64]) -> bool {
            let caller = self.env().caller();
            assert!(self._check_sr25519_sign(*caller.as_ref(), msg, sign), "wrong sign");
            true
        }

        #[ink(message)]
        pub fn check_sr25519_bytes_sign(&mut self, msg: [u8; 32], sign: [u8; 64]) -> bool {
            let caller = self.env().caller();
            assert!(self._check_sr25519_bytes_sign(*caller.as_ref(), msg, sign), "wrong sign");
            true
        }

        pub fn _check_sr25519_sign(&self, public: [u8; 32], msg: [u8; 32], sign: [u8; 64]) -> bool {
            let res = self.env().extension().verify_sr25519(public, msg, sign);
            if res.is_ok() {
                return true;
            }

            return false
        }

        const bytes_pre: [char; 7] = ['<', 'B', 'y', 't', 'e', 's', '>'];
        const bytes_sub: [char; 8] = ['<', '/', 'B', 'y', 't', 'e', 's', '>'];
        pub fn _check_sr25519_bytes_sign(&self, public: [u8; 32], msg: [u8; 32], sign: [u8; 64]) -> bool {
            let mut tmp = [0; 47];
            let mut index = 0;
            for ch in Polkasign::bytes_pre {
                tmp[index.clone()] = ch as u8;
                index += 1;
            }
            for ch in msg {
                tmp[index.clone()] = ch;
                index += 1;
            }
            for ch in Polkasign::bytes_sub {
                tmp[index.clone()] = ch as u8;
                index += 1;
            }
            let res = self.env().extension().verify_sr25519_bytes(public, tmp, sign);
            if res.is_ok() {
                return true;
            }

            return false
        }

        #[ink(message)]
        pub fn query_agreement_by_creator(&mut self, creator: AccountId, pageParams: PageParams) -> PageResult<AgreementInfoDisplay> {
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
                result.push(Polkasign::convAgreement2Display(self.agreements_map.get(&ids[i as usize]).unwrap()));
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
        pub fn query_agreement_by_id(&mut self, index: u64) -> AgreementInfoDisplay {
            let a = self.agreements_map.get(&index).unwrap();
            Polkasign::convAgreement2Display(a)
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        pub fn index(&self) -> u64 {
            self.index.clone()
        }

        fn convAgreement2Display(a: &AgreementInfo) -> AgreementInfoDisplay {
            let sign_infos = a.sign_infos.values().cloned().collect();
            let mut resources: Vec<StorageInfo> = Vec::new();
            let res: Vec<Vec<StorageInfo>> = a.resources.values().cloned().collect();
            for item in res {
                for info in item {
                    resources.push(info);
                }
            }

            AgreementInfoDisplay {
                index: a.index,
                creator: a.creator,
                name: a.name.clone(),
                create_at: a.create_at,
                status: a.status,
                signers: a.signers.clone(),
                agreement_file: a.agreement_file.clone(),
                sign_infos,
                resources
            }
        }

        #[ink(message)]
        pub fn query_agreement_by_collaborator(&mut self, collaborator: AccountId, pageParams: PageParams) -> PageResult<AgreementInfoDisplay> {
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
                result.push(Polkasign::convAgreement2Display(self.agreements_map.get(&ids[i as usize]).unwrap()));
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
        use std::convert::TryFrom;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        #[ink::test]
        fn new_works() {
            let test_account :AccountId = [0u8; 32].into();
            let polkasion = Polkasign::new(test_account);
            assert_eq!(polkasion.index(), 0);
            assert_eq!(polkasion.owner(), test_account);
        }

        #[ink::test]
        fn create_agreement_and_query_agreement_by_id() {
            let test_account :AccountId = [0u8; 32].into();
            let mut polkasion = Polkasign::new(test_account);
            let params = CreateAgreementParams {
                name: "test".to_string(),
                signers: vec![[1u8; 32].into()],
                agreement_file: StorageInfo {
                    hash: [7u8; 32].into(),
                    creator: [1u8; 32].into(),
                    usage: "doc".to_string(),
                    save_at: "ipfs".to_string(),
                    url: "http://ipfs.io/xxxx".to_string()
                }
            };
            let index = polkasion.create_agreement(params.clone());
            assert_eq!(polkasion.index(), index + 1);
            let res = polkasion.query_agreement_by_id(index);
            assert_eq!(res.name, params.name);
            assert_eq!(res.signers, params.signers);
        }

        #[ink::test]
        fn create_agreement_and_query_agreement_by_creator() {
            let test_account :AccountId = [0u8; 32].into();
            let mut polkasion = Polkasign::new(test_account);
            let params = CreateAgreementParams {
                name: "test".to_string(),
                signers: vec![[1u8; 32].into()],
                agreement_file: StorageInfo {
                    hash: [7u8; 32].into(),
                    creator: [1u8; 32].into(),
                    usage: "doc".to_string(),
                    save_at: "ipfs".to_string(),
                    url: "http://ipfs.io/xxxx".to_string()
                }
            };
            let index = polkasion.create_agreement(params.clone());
            assert_eq!(polkasion.index(), index + 1);
            let res = polkasion.query_agreement_by_creator([1u8; 32].into(), PageParams{
                page_index: 0,
                page_size: 10,
            });
            assert_eq!(res.total, 1);
            assert_eq!(res.data[0].name, params.name);
            assert_eq!(res.data[0].signers, params.signers);
        }

        #[ink::test]
        fn create_agreement_and_query_agreement_by_collaborator() {
            let test_account :AccountId = [0u8; 32].into();
            let mut polkasion = Polkasign::new(test_account);
            let params = CreateAgreementParams {
                name: "test".to_string(),
                signers: vec![[1u8; 32].into(), [2u8; 32].into()],
                agreement_file: StorageInfo {
                    hash: [7u8; 32].into(),
                    creator: [1u8; 32].into(),
                    usage: "doc".to_string(),
                    save_at: "ipfs".to_string(),
                    url: "http://ipfs.io/xxxx".to_string()
                }
            };
            let index = polkasion.create_agreement(params.clone());
            assert_eq!(polkasion.index(), index + 1);
            let res = polkasion.query_agreement_by_collaborator([2u8; 32].into(), PageParams{
                page_index: 0,
                page_size: 10,
            });
            assert_eq!(res.total, 1);
            assert_eq!(res.data[0].name, params.name);
            assert_eq!(res.data[0].signers, params.signers);
        }

        #[ink::test]
        fn attach_resource_and_query_agreement_by_id() {
            let test_account :AccountId = [0u8; 32].into();
            let mut polkasion = Polkasign::new(test_account);
            let params = CreateAgreementParams {
                name: "test".to_string(),
                signers: vec![[1u8; 32].into()],
                agreement_file: StorageInfo {
                    hash: [7u8; 32].into(),
                    creator: [1u8; 32].into(),
                    usage: "doc".to_string(),
                    save_at: "ipfs".to_string(),
                    url: "http://ipfs.io/xxxx".to_string()
                }
            };
            let index = polkasion.create_agreement(params.clone());
            assert_eq!(polkasion.index(), index + 1);

            // attach resource
            let info = StorageInfo {
                hash: [2u8; 32].into(),
                creator: [1u8; 32].into(),
                usage: "comment".to_string(),
                save_at: "ipfs".to_string(),
                url: "https://ipfs.io/xxxx".to_string()
            };
            polkasion.attach_resource_to_agreement(index, info.clone());

            let res = polkasion.query_agreement_by_id(index);
            assert_eq!(res.name, params.name);
            assert_eq!(res.signers, params.signers);
            assert_eq!(res.resources[0].hash, info.hash);
        }
    }
}
