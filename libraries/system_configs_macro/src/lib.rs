/*!
 * AssetsDAO
 * https://github.com/RandSeedOrg/AssetsDAO
 * Copyright (C) 2025 RandSeedOrg
 * https://github.com/RandSeedOrg/AssetsDAO/blob/master/LICENSE
 */

use proc_macro::TokenStream;

#[proc_macro]
pub fn system_configs(_: TokenStream) -> TokenStream {
  quote::quote! {
    pub mod system_configs {
      use ic_cdk::{api::{is_controller, msg_caller}};
      use std::{cell::RefCell, collections::{HashMap, HashSet}};
      use candid::Principal;
      use types::{sys::{config::{PermissionCode, RoleCode, SystemConfig, UserRolePermissionVo}, dict::{transfer_structures::DictVo, DictCode}, ExteralCanisterLabels, SystemSwitches}, UserId};
      
      thread_local! {
        // cache the user role permissions
        static USER_ROLE_PERMISSIONS: RefCell<HashMap<UserId, (bool, HashSet<RoleCode>, HashSet<PermissionCode>)>> = RefCell::new(
          HashMap::new()
        );
      
        // cache the dictionary
        static DICT: RefCell<HashMap<DictCode, DictVo>> = RefCell::new(
          HashMap::new()
        );
      }
      
      #[ic_cdk::update]
      pub async fn setup_subscribe(admin: Principal) -> String {
        if !caller_is_controller().await {
          return String::from("Setup subscribe failed: caller is not a controller!");
        }
      
        match ic_cdk::call::Call::unbounded_wait(admin, "subscribe").await {
          Ok(_) => {
            String::from("Subscribed successfully")
          }
          Err(err) => {
            format!("Failed to subscribe {:?}", err)
          }
        }
      }
      
      #[ic_cdk::update]
      pub async fn update_system_configs(config: SystemConfig) -> () {
        if !caller_is_controller().await {
          ic_cdk::trap("Update system configs failed : caller is not a controller!");
        }
      
        USER_ROLE_PERMISSIONS.with(|cache| {
          cache.borrow_mut().clear();
          for user_role_permission in config.user_role_permissions {
            cache.borrow_mut().insert(user_role_permission.principal_id.clone(), (user_role_permission.is_controller, user_role_permission.role_codes.iter().cloned().collect(), user_role_permission.permission_codes.iter().cloned().collect()));
          }
        });
      
        DICT.with(|cache| {
          cache.borrow_mut().clear();
          for dict in config.dicts {
            cache.borrow_mut().insert(dict.code.clone(), dict);
          }
        });
      
        ic_cdk::api::debug_print(format!("System configs updated successfully"));
      }
      
      
      #[ic_cdk::query]
      pub fn get_user_role_permissions() -> Vec<UserRolePermissionVo> {
        USER_ROLE_PERMISSIONS.with(|cache| {
          cache.borrow().iter().map(|(principal_id, (is_controller, role_codes, permission_codes))| {
            UserRolePermissionVo {
              principal_id: principal_id.clone(),
              is_controller: *is_controller,
              role_codes: role_codes.iter().cloned().collect(),
              permission_codes: permission_codes.iter().cloned().collect(),
            }
          }).collect()
        })
      }
      
      pub fn get_dict_with_dict_code(dict_code: &DictCode) -> Option<DictVo> {
        DICT.with(|cache| {
          cache.borrow().get(dict_code).cloned()
        })
      }
      
      /// Check if the caller has the given permission
      pub fn has_permission(permission_code: &str) -> bool {
        let caller = msg_caller().to_text();
        USER_ROLE_PERMISSIONS.with(|cache| {
          match cache.borrow().get(&caller) {
            Some((_, _, permission_codes)) => permission_codes.contains(permission_code),
            None => false
          }
        })
      }
      
      pub fn get_exteral_canister_id(canister: ExteralCanisterLabels) -> Principal {
        let dict = match get_dict_with_dict_code(&DictCode::from("system_config")) {
          Some(dict) => dict,
          None => ic_cdk::trap("Failed to get system config dict!")
        };
      
        let canister_item = dict.items.iter().find(|item| item.label == canister.to_string());
      
        match canister_item {
          Some(item) => Principal::from_text(&item.value).unwrap_or_else(|_| ic_cdk::trap(&format!("Failed to get canister id: canister label = {} and value = {} is not a valid principal!", canister.to_string(), item.value)),),
          None => ic_cdk::trap(&format!("Failed to get canister id: canister label = {} not found!", canister.to_string()))
        }
      }

      pub fn system_switch_is_open(switch: SystemSwitches) -> bool {
        let dict = match get_dict_with_dict_code(&DictCode::from("system_switches")) {
          Some(dict) => dict,
          None => ic_cdk::trap("Failed to get system switches dict!")
        };
      
        let canister_item = dict.items.iter().find(|item| item.label == switch.to_string());
      
        match canister_item {
          Some(item) => item.value == "1",
          None => false
        }
      }

      pub fn system_switch_is_close(switch: SystemSwitches) -> bool {
        !system_switch_is_open(switch)
      }
      
      pub async fn caller_is_controller() -> bool {
        is_controller(&msg_caller())
      }
    }
  }
  .into()
}


#[proc_macro_attribute]
pub fn has_permission(attr: TokenStream, item: TokenStream) -> TokenStream {
  let attr_input = syn::parse_macro_input!(attr as syn::LitStr);
  let input = syn::parse_macro_input!(item as syn::ItemFn);
  let name = &input.sig.ident;
  let block = &input.block;
  let vis = &input.vis;
  let attrs = &input.attrs;
  let asyncness = &input.sig.asyncness;
  let inputs = &input.sig.inputs;
  let output = &input.sig.output;

  quote::quote! {
    #(#attrs)*
    #vis #asyncness fn #name(#inputs) #output {
      if !crate::system_configs::has_permission(#attr_input) {
        ic_cdk::trap(&format!("Caller does not have permission to call {}", stringify!(#attr_input)));
      }
      #block
    }
  }
  .into()
}

#[proc_macro_attribute]
pub fn has_permission_option(attr: TokenStream, item: TokenStream) -> TokenStream {
  let attr_input = syn::parse_macro_input!(attr as syn::LitStr);
  let input = syn::parse_macro_input!(item as syn::ItemFn);
  let name = &input.sig.ident;
  let block = &input.block;
  let vis = &input.vis;
  let attrs = &input.attrs;
  let asyncness = &input.sig.asyncness;
  let inputs = &input.sig.inputs;
  let output = &input.sig.output;

  quote::quote! {
    #(#attrs)*
    #vis #asyncness fn #name(#inputs) #output {
      if !crate::system_configs::has_permission(#attr_input) {
        return Option::Some(format!("Caller does not have permission to call {}", stringify!(#attr_input)));
      }
      #block
    }
  }
  .into()
}

#[proc_macro_attribute]
pub fn has_permission_result(attr: TokenStream, item: TokenStream) -> TokenStream {
  let attr_input = syn::parse_macro_input!(attr as syn::LitStr);
  let input = syn::parse_macro_input!(item as syn::ItemFn);
  let name = &input.sig.ident;
  let block = &input.block;
  let vis = &input.vis;
  let attrs = &input.attrs;
  let asyncness = &input.sig.asyncness;
  let inputs = &input.sig.inputs;
  let output = &input.sig.output;

  quote::quote! {
    #(#attrs)*
    #vis #asyncness fn #name(#inputs) #output {
      if !crate::system_configs::has_permission(#attr_input) {
        return Err(format!("Caller does not have permission to call {}", stringify!(#attr_input)));
      }
      #block
    }
  }
  .into()
}