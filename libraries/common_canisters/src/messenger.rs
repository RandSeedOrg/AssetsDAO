// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports, deprecated)]
use candid::{self, CandidType, Deserialize, Principal, Encode, Decode};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
pub enum Result_ { Ok(u64), Err(String) }

#[derive(CandidType, Deserialize)]
pub enum Result1 { Ok, Err(String) }

#[derive(CandidType, Deserialize)]
pub struct MessageCommon { pub content: String }

#[derive(CandidType, Deserialize)]
pub struct NotificationInfo {
  pub title: String,
  pub links: Option<Vec<String>>,
  pub notification_type: Option<String>,
  pub avatar: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub enum SendingMethod { Email, Comment, InApp, Browser }

#[derive(CandidType, Deserialize)]
pub struct CreateMessageDto {
  pub msg: MessageCommon,
  pub notification_info: Option<NotificationInfo>,
  pub sending_method: Vec<SendingMethod>,
}

#[derive(CandidType, Deserialize)]
pub enum Result2 { Ok(Vec<u64>), Err(String) }

#[derive(CandidType, Deserialize)]
pub struct CreateNotificationDto {
  pub title: String,
  pub content: String,
  pub links: Option<Vec<String>>,
  pub notification_type: String,
  pub sending_method: Option<Vec<SendingMethod>>,
  pub avatar: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct MetaData {
  pub updated_at: Option<u64>,
  pub updated_by: Option<String>,
  pub created_at: Option<u64>,
  pub created_by: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct BaseMessage {
  pub content: Option<String>,
  pub meta: Option<MetaData>,
}

#[derive(CandidType, Deserialize)]
pub struct Notification {
  pub msg: Option<BaseMessage>,
  pub title: Option<String>,
  pub published: Option<bool>,
  pub links: Option<Vec<String>>,
  pub notification_type: Option<String>,
  pub sending_method: Option<Vec<SendingMethod>>,
  pub avatar: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct AllNotificationDetailsVo {
  pub page_size: u64,
  pub total: u64,
  pub page: u64,
  pub total_pages: u64,
  pub details: Vec<(u64,Notification,)>,
}

#[derive(CandidType, Deserialize)]
pub enum Result3 { Ok(AllNotificationDetailsVo), Err(String) }

#[derive(CandidType, Deserialize)]
pub enum PropertyValue { U64(u64), Text(String) }

#[derive(CandidType, Deserialize)]
pub struct PropertiesDto { pub prop: PropertyValue, pub timestamp: u64 }

#[derive(CandidType, Deserialize)]
pub struct BadgeNameWithPropsVo {
  pub name: String,
  pub props: Option<PropertiesDto>,
}

#[derive(CandidType, Deserialize)]
pub enum Result4 { Ok(Vec<(String,Vec<BadgeNameWithPropsVo>,)>), Err(String) }

#[derive(CandidType, Deserialize)]
pub struct EmailConfigs {
  pub count_reset_interval: Option<u64>,
  pub api_key: Option<String>,
  pub min_interval_global: Option<u64>,
  pub max_send_count_total: Option<u32>,
  pub template_id: Option<String>,
  pub verify_length: Option<u64>,
  pub app_id: Option<String>,
  pub max_send_count: Option<u16>,
  pub expiration_time: Option<u64>,
  pub min_interval: Option<u64>,
  pub modify_reset_after: Option<u16>,
}

#[derive(CandidType, Deserialize)]
pub enum Result5 { Ok(EmailConfigs), Err(String) }

#[derive(CandidType, Deserialize)]
pub struct NotificationDetailVo {
  pub title: Option<String>,
  pub content: Option<String>,
  pub meta: Option<MetaData>,
  pub published: Option<bool>,
  pub links: Option<Vec<String>>,
  pub notification_type: Option<String>,
  pub sending_method: Option<Vec<SendingMethod>>,
  pub avatar: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub enum Result6 { Ok(NotificationDetailVo), Err(String) }

#[derive(CandidType, Deserialize)]
pub struct TypeAvatarPair {
  pub notify_type: Option<String>,
  pub avatar: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct NotifyConfigs {
  pub type_avatar_pairs: Option<Vec<TypeAvatarPair>>,
  pub content_max_length: Option<u64>,
  pub title_max_length: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct PublishHistoryVo {
  pub read_count: u64,
  pub canceled: bool,
  pub target_users: Vec<Principal>,
  pub publisher: String,
  pub notify_id: u64,
  pub publish_id: u64,
  pub total_sent: u64,
  pub publish_time: u64,
  pub target_tags: Vec<String>,
}

#[derive(CandidType, Deserialize)]
pub struct PublishHistoryPage {
  pub page_size: u64,
  pub total: u64,
  pub data: Vec<PublishHistoryVo>,
  pub page: u64,
  pub total_pages: u64,
}

#[derive(CandidType, Deserialize)]
pub enum Result7 { Ok(PublishHistoryPage), Err(String) }

#[derive(CandidType, Deserialize)]
pub enum Result8 { Ok(String), Err(String) }

#[derive(CandidType, Deserialize)]
pub enum Result9 { Ok(Vec<BadgeNameWithPropsVo>), Err(String) }

#[derive(CandidType, Deserialize)]
pub struct UserInfo {
  pub user_name: String,
  pub registration_time: u64,
  pub email: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct NotificationSummary {
  pub is_read: bool,
  pub title: String,
  pub content: String,
  pub create_at: u64,
  pub publish_id: u64,
  pub links: Option<Vec<String>>,
  pub notification_id: u64,
  pub notify_type: String,
  pub is_deleted: bool,
  pub avatar: Option<String>,
  pub publish_time: u64,
}

#[derive(CandidType, Deserialize)]
pub struct NotificationSummaryVo {
  pub page_size: u64,
  pub deleted: u64,
  pub total: u64,
  pub page: u64,
  pub total_pages: u64,
  pub unread: u64,
  pub kept_notification_summaries: Vec<NotificationSummary>,
}

#[derive(CandidType, Deserialize)]
pub enum Result10 { Ok(NotificationSummaryVo), Err(String) }

#[derive(CandidType, Deserialize)]
pub struct UserRolePermissionVo {
  pub permission_codes: Vec<String>,
  pub principal_id: String,
  pub role_codes: Vec<String>,
  pub is_controller: bool,
}

#[derive(CandidType, Deserialize)]
pub struct UserNotificationKey { pub notify_id: u64, pub publish_id: u64 }

#[derive(CandidType, Deserialize)]
pub enum Result11 { Ok(bool), Err(String) }

#[derive(CandidType, Deserialize)]
pub struct PublishNotificationDto {
  pub target_users: Option<Vec<Principal>>,
  pub notification_id: u64,
  pub target_tags: Option<Vec<String>>,
}

#[derive(CandidType, Deserialize)]
pub struct DailyTotal { pub count: u32, pub last_reset_date: u64 }

#[derive(CandidType, Deserialize)]
pub struct VerifyInfo {
  pub times: Option<u16>,
  pub code: Option<String>,
  pub timestamp: Option<u64>,
  pub email_id: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum Result12 { Ok(VerifyInfo), Err(String) }

#[derive(CandidType, Deserialize)]
pub struct LatestVerifyResult {
  pub email: Option<String>,
  pub scheduled_reset_time: Option<u64>,
  pub timestamp: Option<u64>,
  pub modify: Option<bool>,
}

#[derive(CandidType, Deserialize)]
pub enum Result13 { Ok(LatestVerifyResult), Err(String) }

#[derive(CandidType, Deserialize)]
pub struct VerifyEmailVo {
  pub principal: Option<String>,
  pub recipient: Option<String>,
  pub template_id: Option<String>,
  pub verify_code: String,
  pub timestamp: u64,
  pub email_id: u64,
  pub origin_id: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum Result14 { Ok(VerifyEmailVo), Err(String) }

#[derive(CandidType, Deserialize)]
pub enum Result15 { Ok(u64,Vec<(String,u16,)>,), Err(String) }

#[derive(CandidType, Deserialize)]
pub enum Result16 { Ok(Vec<VerifyEmailVo>), Err(String) }

#[derive(CandidType, Deserialize)]
pub struct HttpHeader { pub value: String, pub name: String }

#[derive(CandidType, Deserialize)]
pub struct HttpResponse {
  pub status: candid::Nat,
  pub body: serde_bytes::ByteBuf,
  pub headers: Vec<HttpHeader>,
}

#[derive(CandidType, Deserialize)]
pub struct TransformArgs {
  pub context: serde_bytes::ByteBuf,
  pub response: HttpResponse,
}

#[derive(CandidType, Deserialize)]
pub struct EmailConfigsUpdateDto {
  pub count_reset_interval: Option<Option<u64>>,
  pub api_key: Option<Option<String>>,
  pub min_interval_global: Option<Option<u64>>,
  pub max_send_count_total: Option<Option<u32>>,
  pub template_id: Option<Option<String>>,
  pub app_id: Option<Option<String>>,
  pub max_send_count: Option<Option<u16>>,
  pub expiration_time: Option<Option<u64>>,
  pub min_interval: Option<Option<u64>>,
  pub modify_reset_after: Option<Option<u16>>,
}

#[derive(CandidType, Deserialize)]
pub struct UpdateNotificationDto {
  pub title: Option<String>,
  pub content: Option<String>,
  pub links: Option<Vec<String>>,
  pub sending_method: Option<Vec<SendingMethod>>,
  pub avatar: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct DictItemVo {
  pub value: String,
  pub sort: u16,
  pub description: String,
  pub label: String,
}

#[derive(CandidType, Deserialize)]
pub struct DictVo {
  pub id: u64,
  pub code: String,
  pub name: String,
  pub description: String,
  pub items: Vec<DictItemVo>,
}

#[derive(CandidType, Deserialize)]
pub struct SystemConfig {
  pub dicts: Vec<DictVo>,
  pub user_role_permissions: Vec<UserRolePermissionVo>,
}

#[derive(CandidType, Deserialize)]
pub struct BadgeWithProps { pub badge: u64, pub props: Option<PropertiesDto> }

pub struct Service(pub Principal);
impl Service {
  pub async fn batch_delete_unpublished_notifications(
    &self,
    arg0: Vec<u64>,
  ) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "batch_delete_unpublished_notifications", (
      arg0,
    )).await
  }
  pub async fn cancel_publish(&self, arg0: u64) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "cancel_publish", (arg0,)).await
  }
  pub async fn cleanup_old_data(&self) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "cleanup_old_data", ()).await
  }
  pub async fn clear_uesr_info(&self) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "clear_uesr_info", ()).await
  }
  pub async fn create_message(&self, arg0: CreateMessageDto) -> Result<
    (Result2,)
  > { ic_cdk::call(self.0, "create_message", (arg0,)).await }
  pub async fn create_notification(
    &self,
    arg0: CreateNotificationDto,
  ) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "create_notification", (arg0,)).await
  }
  pub async fn get_all_notifications(
    &self,
    arg0: Option<String>,
    arg1: Option<u64>,
    arg2: Option<u64>,
  ) -> Result<(Result3,)> {
    ic_cdk::call(self.0, "get_all_notifications", (arg0,arg1,arg2,)).await
  }
  pub async fn get_all_user_badges(&self) -> Result<(Result4,)> {
    ic_cdk::call(self.0, "get_all_user_badges", ()).await
  }
  pub async fn get_email_config(&self) -> Result<(Result5,)> {
    ic_cdk::call(self.0, "get_email_config", ()).await
  }
  pub async fn get_notification_detail(&self, arg0: u64) -> Result<(Result6,)> {
    ic_cdk::call(self.0, "get_notification_detail", (arg0,)).await
  }
  pub async fn get_notify_config(&self) -> Result<(NotifyConfigs,)> {
    ic_cdk::call(self.0, "get_notify_config", ()).await
  }
  pub async fn get_publish_history(
    &self,
    arg0: Option<u64>,
    arg1: Option<bool>,
    arg2: Option<u64>,
    arg3: Option<u64>,
  ) -> Result<(Result7,)> {
    ic_cdk::call(self.0, "get_publish_history", (arg0,arg1,arg2,arg3,)).await
  }
  pub async fn get_uesr_name(&self, arg0: u64) -> Result<(Result8,)> {
    ic_cdk::call(self.0, "get_uesr_name", (arg0,)).await
  }
  pub async fn get_user_badges(&self, arg0: String) -> Result<(Result9,)> {
    ic_cdk::call(self.0, "get_user_badges", (arg0,)).await
  }
  pub async fn get_user_info(&self, arg0: Option<String>) -> Result<
    (Vec<(String,UserInfo,)>,)
  > { ic_cdk::call(self.0, "get_user_info", (arg0,)).await }
  pub async fn get_user_notifications(
    &self,
    arg0: Option<String>,
    arg1: Option<u64>,
    arg2: Option<u64>,
  ) -> Result<(Result10,)> {
    ic_cdk::call(self.0, "get_user_notifications", (arg0,arg1,arg2,)).await
  }
  pub async fn get_user_role_permissions(&self) -> Result<
    (Vec<UserRolePermissionVo>,)
  > { ic_cdk::call(self.0, "get_user_role_permissions", ()).await }
  pub async fn import_user_info(&self, arg0: Vec<(String,UserInfo,)>) -> Result<
    (Result1,)
  > { ic_cdk::call(self.0, "import_user_info", (arg0,)).await }
  pub async fn mark_as_delete(
    &self,
    arg0: Option<Vec<UserNotificationKey>>,
  ) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "mark_as_delete", (arg0,)).await
  }
  pub async fn mark_as_read(
    &self,
    arg0: Option<Vec<UserNotificationKey>>,
  ) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "mark_as_read", (arg0,)).await
  }
  pub async fn modify_email_update_badge(
    &self,
    arg0: String,
    arg1: String,
  ) -> Result<(Result11,)> {
    ic_cdk::call(self.0, "modify_email_update_badge", (arg0,arg1,)).await
  }
  pub async fn publish_notification(
    &self,
    arg0: PublishNotificationDto,
  ) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "publish_notification", (arg0,)).await
  }
  pub async fn query_daily_total(&self) -> Result<(DailyTotal,)> {
    ic_cdk::call(self.0, "query_daily_total", ()).await
  }
  pub async fn query_intermediate_verify_info(&self, arg0: String) -> Result<
    (Result12,)
  > { ic_cdk::call(self.0, "query_intermediate_verify_info", (arg0,)).await }
  pub async fn query_latest_verify_result(&self, arg0: String) -> Result<
    (Result13,)
  > { ic_cdk::call(self.0, "query_latest_verify_result", (arg0,)).await }
  pub async fn query_verify_email_by_id(&self, arg0: u64) -> Result<
    (Result14,)
  > { ic_cdk::call(self.0, "query_verify_email_by_id", (arg0,)).await }
  pub async fn query_verify_email_statistics(
    &self,
    arg0: Option<u64>,
    arg1: Option<u64>,
  ) -> Result<(Result15,)> {
    ic_cdk::call(self.0, "query_verify_email_statistics", (arg0,arg1,)).await
  }
  pub async fn query_verify_history(&self, arg0: String) -> Result<
    (Result16,)
  > { ic_cdk::call(self.0, "query_verify_history", (arg0,)).await }
  pub async fn resend(&self, arg0: String) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "resend", (arg0,)).await
  }
  pub async fn send_http_post_request(
    &self,
    arg0: String,
    arg1: String,
    arg2: String,
    arg3: String,
  ) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "send_http_post_request", (arg0,arg1,arg2,arg3,)).await
  }
  pub async fn send_verify_email(
    &self,
    arg0: Option<u64>,
    arg1: Option<String>,
    arg2: Option<String>,
    arg3: String,
  ) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "send_verify_email", (arg0,arg1,arg2,arg3,)).await
  }
  pub async fn setup_subscribe(&self, arg0: Principal) -> Result<(String,)> {
    ic_cdk::call(self.0, "setup_subscribe", (arg0,)).await
  }
  pub async fn transform(&self, arg0: TransformArgs) -> Result<
    (HttpResponse,)
  > { ic_cdk::call(self.0, "transform", (arg0,)).await }
  pub async fn update_email_config(
    &self,
    arg0: EmailConfigsUpdateDto,
  ) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "update_email_config", (arg0,)).await
  }
  pub async fn update_notification(
    &self,
    arg0: u64,
    arg1: UpdateNotificationDto,
  ) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "update_notification", (arg0,arg1,)).await
  }
  pub async fn update_notify_config(&self, arg0: NotifyConfigs) -> Result<
    (Result1,)
  > { ic_cdk::call(self.0, "update_notify_config", (arg0,)).await }
  pub async fn update_system_configs(&self, arg0: SystemConfig) -> Result<()> {
    ic_cdk::call(self.0, "update_system_configs", (arg0,)).await
  }
  pub async fn update_user_badges(
    &self,
    arg0: String,
    arg1: u64,
    arg2: bool,
    arg3: Option<Vec<BadgeWithProps>>,
  ) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "update_user_badges", (arg0,arg1,arg2,arg3,)).await
  }
  pub async fn verify_code(&self, arg0: String) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "verify_code", (arg0,)).await
  }
}
