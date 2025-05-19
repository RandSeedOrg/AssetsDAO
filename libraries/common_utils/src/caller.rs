use candid::Principal;

/// Reject anonymous calls
/// This function checks if the caller is anonymous and traps the execution if it is.
/// It is used to prevent anonymous calls to the canister.
pub fn reject_anonymous_call() {
  if ic_cdk::api::msg_caller() == Principal::anonymous() {
    ic_cdk::trap("call rejected: anonymous caller");
  }
}
