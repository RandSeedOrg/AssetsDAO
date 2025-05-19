pub mod nns_governance_api;

use candid::Principal;
use nns_governance_api::Service;

pub fn get_governance() -> Service {
  Service(Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap())
}
