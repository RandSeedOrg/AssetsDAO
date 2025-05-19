/*!
 * AssetsDAO
 * https://github.com/RandSeedOrg/AssetsDAO
 * Copyright (C) 2025 RandSeedOrg
 * https://github.com/RandSeedOrg/AssetsDAO/blob/master/LICENSE
 */

use proc_macro::TokenStream;

#[proc_macro]
pub fn parallel_guard(_: TokenStream) -> TokenStream {
  quote::quote! {
    pub mod parallel_guard {
      use std::cell::RefCell;
      use std::collections::HashSet;

      thread_local! {
        // cache the user role permissions
        static ENTRY_STATE: RefCell<HashSet<String>> = RefCell::new(
          HashSet::new()
        );
      }

      pub struct EntryGuard {
        item: String,
      }

      #[derive(Debug, PartialEq, Eq)]
      pub enum EntryGuardError {
        AlreadyProcessing,
      }

      impl EntryGuard {
        pub fn new(item: String) -> Result<Self, EntryGuardError> {
          ENTRY_STATE.with(|state| {
            if !state.borrow_mut().insert(item.clone()) {
              return Err(EntryGuardError::AlreadyProcessing);
            }
            Ok(Self { item })
          })
        }
      }

      impl Drop for EntryGuard {
        fn drop(&mut self) {
          ENTRY_STATE.with(|state| {
            state.borrow_mut().remove(&self.item);
          });
        }
      }
    }
  }
  .into()
}
