// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Utilities for testing if the storage interaction of an object
//! which is pushed/pulled/cleared to/from storage behaves as it should.

/// Runs `f` using the off-chain testing environment.
#[cfg(test)]
pub fn run_test<F>(f: F)
where
    F: FnOnce(),
{
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        f();
        Ok(())
    })
    .unwrap()
}

/// Creates two tests:
/// (1) Tests if an object which is `push_spread`-ed to storage results in exactly
///     the same object when it is `pull_spread`-ed again. Subsequently the object
///     undergoes the same test for `push_packed` and `pull_packed`.
/// (2) Tests if `clear_spread` removes the object properly from storage.
#[macro_export]
macro_rules! push_pull_works_for_primitive {
    ( $name:ty, [$($value:expr),*] ) => {
        paste::item! {
            #[test]
            #[allow(non_snake_case)]
            fn [<$name _pull_push_works>] () {
                crate::test_utils::run_test(|| {
                    $({
                        let x: $name = $value;
                        let key = ink_primitives::Key::from([0x42; 32]);
                        let key2 = ink_primitives::Key::from([0x77; 32]);
                        crate::traits::push_spread_root(&x, &key);
                        let y: $name = crate::traits::pull_spread_root(&key);
                        assert_eq!(x, y);
                        crate::traits::push_packed_root(&x, &key2);
                        let z: $name = crate::traits::pull_packed_root(&key2);
                        assert_eq!(x, z);
                    })*
                })
            }

            #[test]
            #[should_panic(expected = "storage entry was empty")]
            #[allow(non_snake_case)]
            fn [<$name _clean_works>]() {
                crate::test_utils::run_test(|| {
                    $({
                        let x: $name = $value;
                        let key = ink_primitives::Key::from([0x42; 32]);
                        crate::traits::push_spread_root(&x, &key);
                        // Works since we just populated the storage.
                        let y: $name = crate::traits::pull_spread_root(&key);
                        assert_eq!(x, y);
                        crate::traits::clear_spread_root(&x, &key);
                        // Panics since it loads eagerly from cleared storage.
                        let _: $name = crate::traits::pull_spread_root(&key);
                    })*
                })
            }
        }
    };
}

/// A trait to enable running some fuzz tests on a collection.
pub trait FuzzCollection {
    type Collection;
    type Item;

    /// Executes a series of operations on `self` in order to make it
    /// equal to `template`.
    fn equalize(&mut self, template: &Self::Collection);

    /// Takes a value from `self` and puts it into `item`.
    fn assign(&mut self, item: Self::Item);
}

/// Creates two fuzz tests. Both tests have the same flow:
///     - Take two instances of the collection, generated by our fuzzer
///     - Push `instance2` to storage, pull it out and assert that what
///       is pulled out is what was pushed.
///     - Do some mutations on the `pulled` object. Here the two tests
///       behave differently:
///
///         * `fuzz_ $id _mutate_some` Mutates some entries of the data
///           structure based on the content of `instance2`.
///
///         * `fuzz_ $id _mutate_all` Mutates the entire data structure,
///           so that it has the same content as `instance2`.
///
///     - Push the mutated `pulled` object into storage again, pull it
///       out as `pulled2` and assert that both objects are equal.
///     - Clear the object from storage and assert that storage was
///       cleared up properly, without any leftovers.
#[macro_export]
macro_rules! fuzz_storage {
    ($id:literal, $collection_type:ty) => {
        ::paste::paste! {
            /// Does some basic storage interaction tests whilst mutating
            /// *some* of the data structure's entries.
            #[allow(trivial_casts)]
            #[quickcheck]
            fn [< fuzz_ $id _mutate_some >] (
                instance1: $collection_type,
                mut instance2: $collection_type,
            ) {
                ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                    // we push the generated object into storage
                    let root_key = ink_primitives::Key::from([0x42; 32]);
                    let ptr = KeyPtr::from(root_key);
                    crate::traits::push_spread_root(&instance1, &mut root_key.clone());

                    // we pull what's in storage and assert that this is what was just pushed
                    let mut pulled: $collection_type = crate::traits::pull_spread_root(&root_key.clone());
                    assert_eq!(instance1, pulled);

                    // we iterate over what was pulled and call `assign` for all entries.
                    // this function may or may not modify elements of `pulled`.
                    pulled.iter_mut().for_each(|item| {
                        // this may leave some entries of `pulled` in `State::Preserved`.
                        // even though the instance which is supposed to be mutated is
                        // `pulled`, we still need to call this on a mutable `instance2`,
                        // since e.g. Vec does a `pop()` in assign, so that we don't always
                        // execute the same operation.
                        (&mut instance2).assign(item);
                    });

                    // we push the `pulled` object, on which we just executed mutations
                    // back into storage and asserts it can be pulled out intact again.
                    crate::traits::push_spread_root(&pulled, &mut root_key.clone());
                    let pulled2: $collection_type = crate::traits::pull_spread_root(&mut root_key.clone());
                    assert_eq!(pulled, pulled2);

                    // we clear the objects from storage and assert that everything was
                    // removed without any leftovers.
                    SpreadLayout::clear_spread(&pulled2, &mut ptr.clone());
                    SpreadLayout::clear_spread(&pulled, &mut ptr.clone());
                    crate::test_utils::assert_storage_clean();

                    Ok(())
                })
                    .unwrap()
            }

            /// Does some basic storage interaction tests whilst mutating
            /// *all* the data structure's entries.
            #[allow(trivial_casts)]
            #[quickcheck]
            fn [< fuzz_ $id _mutate_all >] (
                instance1: $collection_type,
                instance2: $collection_type,
            ) {
                ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                    // we push the generated object into storage
                    let root_key = ink_primitives::Key::from([0x42; 32]);
                    let ptr = KeyPtr::from(root_key);
                    crate::traits::push_spread_root(&instance1, &mut root_key.clone());

                    // we pull what's in storage and assert that this is what was just pushed
                    let mut pulled: $collection_type = crate::traits::pull_spread_root(&root_key.clone());
                    assert_eq!(instance1, pulled);

                    // `pulled` is going to be equalized to `
                    (&mut pulled).equalize(&instance2);

                    // we push the `pulled` object, on which we just executed mutations
                    // back into storage and assert it can be pulled out intact again and
                    // is equal to `instance2`.
                    crate::traits::push_spread_root(&pulled, &mut root_key.clone());
                    let pulled2: $collection_type = crate::traits::pull_spread_root(&mut root_key.clone());
                    assert_eq!(pulled, pulled2);
                    assert_eq!(pulled2, instance2);

                    // we clear the objects from storage and assert that everything was
                    // removed without any leftovers.
                    SpreadLayout::clear_spread(&pulled2, &mut ptr.clone());
                    SpreadLayout::clear_spread(&pulled, &mut ptr.clone());
                    crate::test_utils::assert_storage_clean();

                    Ok(())

                })
                    .unwrap()
            }
        }
    };
}

/// Asserts that the storage is empty, without any leftovers.
#[cfg(all(
    test,
    feature = "ink-fuzz-tests",
    not(feature = "ink-experimental-engine")
))]
pub fn assert_storage_clean() {
    let contract_id =
        ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("contract id must exist");
    let used_cells =
        ink_env::test::count_used_storage_cells::<ink_env::DefaultEnvironment>(
            &contract_id,
        )
        .expect("used cells must be returned");
    assert_eq!(used_cells, 0);
}

/// Asserts that the storage is empty, without any leftovers.
#[cfg(all(test, feature = "ink-fuzz-tests", feature = "ink-experimental-engine"))]
pub fn assert_storage_clean() {
    let contract_id = ink_env::test::callee::<ink_env::DefaultEnvironment>();
    let used_cells =
        ink_env::test::count_used_storage_cells::<ink_env::DefaultEnvironment>(
            &contract_id,
        )
        .expect("used cells must be returned");
    assert_eq!(used_cells, 0);
}
