use proc_macro::TokenStream;

#[proc_macro]
pub fn identity_mapping(_: TokenStream) -> TokenStream {
  quote::quote! {
    pub mod identity_mapping {
      use ic_cdk::{api::{is_controller, msg_caller}};
      use std::{cell::RefCell, collections::HashMap};
      use candid::Principal;

      thread_local! {
        // identity mapping cache
        static IDENTITY_MAPPING: RefCell<HashMap<Principal, Principal>> = RefCell::new(
          HashMap::new()
        );
      }

      #[ic_cdk::update]
      pub async fn subscribe_identity_mapping_change(identity_canister: Principal) -> String {
        if !caller_is_controller().await {
          return String::from("Setup subscribe failed: caller is not a controller!");
        }

        #[allow(deprecated)]
        match ic_cdk::call::<(), ()>(identity_canister, "subscribe_identity_mapping_change", ()).await {
          Ok(_) => {
            String::from("Subscribed successfully")
          }
          Err((code, message)) => {
            format!("Failed to subscribe {:?}, {}", code, message)
          }
        }
      }

      #[ic_cdk::update]
      pub async fn update_identity_mapping(identity_mapping: Vec<(Principal, Principal)>) -> () {
        if !caller_is_controller().await {
          ic_cdk::trap("Update identity mapping failed : caller is not a controller!");
        }

        IDENTITY_MAPPING.with(|cache| {
          for (key, value) in identity_mapping {
            cache.borrow_mut().insert(key, value);
          }
        });

        ic_cdk::api::debug_print(format!("Identity mapping updated successfully"));
      }


      #[ic_cdk::query]
      pub fn query_identity_mapping() -> Vec<(Principal, Principal)> {
        IDENTITY_MAPPING.with(|cache| {
          cache.borrow().iter().map(|(key, value)| (key.clone(), value.clone())).collect()
        })
      }

      pub async fn caller_is_controller() -> bool {
        is_controller(&msg_caller())
      }

      pub fn wl_caller() -> Principal {
        let ic_caller = msg_caller();

        IDENTITY_MAPPING.with(|cache| {
          match cache.borrow().get(&ic_caller) {
            Some(mapped_principal) => mapped_principal.clone(),
            None => ic_caller,
          }
        })
      }
    }
  }
  .into()
}
