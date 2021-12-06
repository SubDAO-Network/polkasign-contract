# polkasign Contract

polkasign Contract is a contract to manager agreements to sign together.

## Modules

### Service
```rust

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

    pub struct SignInfo {
        addr: AccountId,
        create_at: u64,
    }

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

    pub struct CreateAgreementParams {
        name: String,
        signers: Vec<AccountId>,
        agreement_file: StorageInfo,
    }
```

## Interfaces

### instance module
instance module.
```bash
type: tx
definition: pub fn new(owner: AccountId) -> Self;
```

### create agreement
create agreement. add storage info.
```bash
type: tx
definition: pub fn create_agreement(&mut self, params: CreateAgreementParams) -> u64;
```


### create agreement with sign
create agreement with sign.
```bash
type: tx
definition: pub fn create_agreement_with_sign(&mut self, params: CreateAgreementParams, info: StorageInfo, sign: [u8; 64]);
```

### attach resource to agreement
attach resource to agreement to target.
```bash
type: tx
definition: pub fn attach_resource_to_agreement(&mut self, index: u64, info: StorageInfo);
```

### attach resource to agreement with sign
attach resource to agreement to target.
```bash
type: tx
definition: pub fn attach_resource_with_sign(&mut self, index: u64, info: StorageInfo, sign: [u8; 64]);
```

### query agreement by id
query agreement by index.
```bash
type: tx
definition: pub fn query_agreement_by_id(&mut self, index: u64) -> AgreementInfo;
```

### query agreement by creator
query agreement by creator.
```bash
type: tx
definition: pub fn query_agreement_by_creator(&mut self, creator: AccountId, pageParams: PageParams) -> PageResult<AgreementInfo>;
```


### query agreement by collaborator
query agreement by collaborator.
```bash
type: tx
definition: pub fn query_agreement_by_collaborator(&mut self, collaborator: AccountId, pageParams: PageParams) -> PageResult<AgreementInfo>;
```
