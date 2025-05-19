use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::UserId;

use super::dict::transfer_structures::DictVo;

pub type RoleCode = String;
pub type PermissionCode = String;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct UserRolePermissionVo {
  pub principal_id: UserId,
  pub is_controller: bool,
  pub role_codes: Vec<RoleCode>,
  pub permission_codes: Vec<PermissionCode>,
}

impl UserRolePermissionVo {
  pub fn new(principal_id: UserId, is_controller: bool, role_codes: Vec<RoleCode>, permission_codes: Vec<PermissionCode>) -> Self {
    UserRolePermissionVo {
      principal_id,
      is_controller,
      role_codes,
      permission_codes,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct SystemConfig {
  pub user_role_permissions: Vec<UserRolePermissionVo>,
  pub dicts: Vec<DictVo>,
}
