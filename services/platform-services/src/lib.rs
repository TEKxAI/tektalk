use std::{collections::HashSet, sync::Arc};

use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use chrono::Utc;
use dashmap::DashMap;
use tektalk_contracts::v1::{
    account_service_server::AccountService,
    chat_service_server::ChatService,
    consent_management_service_server::ConsentManagementService,
    session_management_service_server::SessionManagementService,
    AccessLevel, ChatMessage, ConsentOwnerType, ConsentRecord, ConsentStatus,
    CreateSessionRequest, CreateSessionResponse, ElevateSessionRequest, ElevateSessionResponse,
    EvaluateConsentRequest, EvaluateConsentResponse, GetMessagesRequest, GetMessagesResponse,
    GrantConsentRequest, GrantConsentResponse, ListConsentsRequest, ListConsentsResponse,
    MessageKind, RevokeConsentRequest, RevokeConsentResponse,
    RevokeSessionRequest, RevokeSessionResponse, SendMessageRequest, SendMessageResponse,
    SignInRequest, SignInResponse, SignUpRequest, SignUpResponse, ValidateSessionRequest,
    ValidateSessionResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;
use tektalk_client_core::id::SnowflakeGenerator;

fn now_ms() -> i64 { Utc::now().timestamp_millis() }
fn required(value: &str, name: &'static str) -> Result<(), Status> {
    if value.trim().is_empty() { Err(Status::invalid_argument(format!("{name} is required"))) } else { Ok(()) }
}

#[derive(Clone)] struct Account { id:String, profile_id:String, password_hash:String, trusted_devices:HashSet<String> }
#[derive(Default)] pub struct AccountServiceImpl { accounts:DashMap<String,Account> }

#[tonic::async_trait]
impl AccountService for AccountServiceImpl {
    async fn sign_up(&self, request:Request<SignUpRequest>)->Result<Response<SignUpResponse>,Status>{
        let r=request.into_inner();required(&r.identifier,"identifier")?;required(&r.display_name,"display_name")?;
        if r.password.len()<10{return Err(Status::invalid_argument("password must contain at least 10 characters"));}
        if self.accounts.contains_key(&r.identifier){return Err(Status::already_exists("identifier is registered"));}
        let account_id=Uuid::new_v4().to_string();let profile_id=Uuid::new_v4().to_string();let device_id=Uuid::new_v4().to_string();
        let salt=SaltString::generate(&mut OsRng);let password_hash=Argon2::default().hash_password(r.password.as_bytes(),&salt).map_err(|_|Status::internal("password hashing failed"))?.to_string();
        self.accounts.insert(r.identifier,Account{id:account_id.clone(),profile_id:profile_id.clone(),password_hash,trusted_devices:HashSet::from([device_id.clone()])});
        Ok(Response::new(SignUpResponse{account_id,profile_id,device_id,access_token:Uuid::new_v4().to_string(),refresh_token:Uuid::new_v4().to_string()}))
    }
    async fn sign_in(&self,request:Request<SignInRequest>)->Result<Response<SignInResponse>,Status>{
        let r=request.into_inner();required(&r.identifier,"identifier")?;let account=self.accounts.get(&r.identifier).ok_or_else(||Status::unauthenticated("invalid credentials"))?;
        let hash=PasswordHash::new(&account.password_hash).map_err(|_|Status::internal("stored credential is invalid"))?;
        if Argon2::default().verify_password(r.password.as_bytes(),&hash).is_err(){return Err(Status::unauthenticated("invalid credentials"));}
        let known=!r.device_id.is_empty()&&account.trusted_devices.contains(&r.device_id);let device_id=if r.device_id.is_empty(){Uuid::new_v4().to_string()}else{r.device_id};
        Ok(Response::new(SignInResponse{account_id:account.id.clone(),profile_id:account.profile_id.clone(),device_id,access_token:if known{Uuid::new_v4().to_string()}else{String::new()},refresh_token:if known{Uuid::new_v4().to_string()}else{String::new()},device_verification_required:!known,challenge_id:if known{String::new()}else{Uuid::new_v4().to_string()}}))
    }
}

pub struct ChatServiceImpl { messages:DashMap<String,Vec<ChatMessage>>,dedup:DashMap<String,i64>,message_ids:Arc<SnowflakeGenerator> }

impl ChatServiceImpl {
    pub fn new(node_id:u16)->Result<Self,tektalk_client_core::id::IdError>{Ok(Self{messages:DashMap::new(),dedup:DashMap::new(),message_ids:Arc::new(SnowflakeGenerator::new(node_id)?)})}
}
impl Default for ChatServiceImpl { fn default()->Self{Self::new(1).expect("default Snowflake node is valid")} }

#[tonic::async_trait]
impl ChatService for ChatServiceImpl {
    async fn send_message(&self,request:Request<SendMessageRequest>)->Result<Response<SendMessageResponse>,Status>{
        let r=request.into_inner();required(&r.conversation_id,"conversation_id")?;required(&r.client_message_id,"client_message_id")?;
        let context=r.context.as_ref().ok_or_else(||Status::unauthenticated("request context is required"))?;required(&context.user_id,"user_id")?;
        validate_message(&r)?;let dedup_key=format!("{}:{}",context.user_id,r.client_message_id);
        if let Some(id)=self.dedup.get(&dedup_key){return Ok(Response::new(SendMessageResponse{client_message_id:r.client_message_id,server_message_id:*id,committed_at_unix_ms:now_ms(),deduplicated:true}));}
        let id=self.message_ids.next_id().map_err(|_|Status::unavailable("message id generation failed"))?;let committed=now_ms();
        self.messages.entry(r.conversation_id.clone()).or_default().push(ChatMessage{server_message_id:id,conversation_id:r.conversation_id,sender_id:context.user_id.clone(),kind:r.kind,text:r.text,media:r.media,sticker_id:r.sticker_id,committed_at_unix_ms:committed});self.dedup.insert(dedup_key,id);
        Ok(Response::new(SendMessageResponse{client_message_id:r.client_message_id,server_message_id:id,committed_at_unix_ms:committed,deduplicated:false}))
    }
    async fn get_messages(&self,request:Request<GetMessagesRequest>)->Result<Response<GetMessagesResponse>,Status>{
        let r=request.into_inner();required(&r.conversation_id,"conversation_id")?;let limit=r.limit.clamp(1,100)as usize;let before=if r.before_message_id<=0{i64::MAX}else{r.before_message_id};
        let mut messages=self.messages.get(&r.conversation_id).map(|v|v.iter().filter(|m|m.server_message_id<before).rev().take(limit).cloned().collect::<Vec<_>>()).unwrap_or_default();messages.reverse();
        let next_before_message_id=messages.first().map(|m|m.server_message_id).unwrap_or(0);Ok(Response::new(GetMessagesResponse{messages,next_before_message_id}))
    }
}

fn validate_message(r:&SendMessageRequest)->Result<(),Status>{
    match MessageKind::try_from(r.kind).unwrap_or(MessageKind::Unspecified){
        MessageKind::Text if r.text.trim().is_empty()=>Err(Status::invalid_argument("text message is empty")),
        MessageKind::Sticker if r.sticker_id.is_empty()=>Err(Status::invalid_argument("sticker_id is required")),
        MessageKind::Voice|MessageKind::Video|MessageKind::Photo if r.media.is_none()=>Err(Status::invalid_argument("media reference is required")),
        MessageKind::Unspecified=>Err(Status::invalid_argument("message kind is required")),
        _=>Ok(())
    }
}

#[derive(Clone)] struct Session { id:String,account_id:String,level:i32,scopes:HashSet<String>,audience:String,token:String,expires_at:i64,revoked:bool }
#[derive(Default)] pub struct SessionManagementServiceImpl { sessions:DashMap<String,Session>,tokens:DashMap<String,String> }

#[tonic::async_trait]
impl SessionManagementService for SessionManagementServiceImpl {
    async fn create_session(&self,request:Request<CreateSessionRequest>)->Result<Response<CreateSessionResponse>,Status>{
        let r=request.into_inner();required(&r.account_id,"account_id")?;required(&r.device_id,"device_id")?;required(&r.audience,"audience")?;validate_level(r.requested_level)?;validate_scopes(r.requested_level,&r.requested_scopes)?;
        let id=Uuid::new_v4().to_string();let token=Uuid::new_v4().to_string();let expires=now_ms()+3_600_000;let scopes=r.requested_scopes.into_iter().collect::<HashSet<_>>();
        self.sessions.insert(id.clone(),Session{id:id.clone(),account_id:r.account_id,level:r.requested_level,scopes:scopes.clone(),audience:r.audience,token:token.clone(),expires_at:expires,revoked:false});self.tokens.insert(token.clone(),id.clone());
        Ok(Response::new(CreateSessionResponse{session_id:id,access_token:token,granted_level:r.requested_level,granted_scopes:scopes.into_iter().collect(),expires_at_unix_ms:expires}))
    }
    async fn validate_session(&self,request:Request<ValidateSessionRequest>)->Result<Response<ValidateSessionResponse>,Status>{
        let r=request.into_inner();let Some(id)=self.tokens.get(&r.access_token).map(|v|v.clone())else{return Ok(Response::new(invalid_session("unknown token")));};let Some(s)=self.sessions.get(&id)else{return Ok(Response::new(invalid_session("unknown session")));};
        let missing=r.required_scopes.iter().any(|scope|!s.scopes.contains(scope));let valid=!s.revoked&&s.expires_at>now_ms()&&s.level>=r.required_level&&s.audience==r.audience&&!missing;
        Ok(Response::new(ValidateSessionResponse{valid,session_id:s.id.clone(),account_id:s.account_id.clone(),granted_level:s.level,granted_scopes:s.scopes.iter().cloned().collect(),failure_reason:if valid{String::new()}else{"level, scope, audience or lifetime check failed".into()}}))
    }
    async fn elevate_session(&self,request:Request<ElevateSessionRequest>)->Result<Response<ElevateSessionResponse>,Status>{
        let r=request.into_inner();required(&r.security_proof,"security_proof")?;validate_level(r.target_level)?;validate_scopes(r.target_level,&r.requested_scopes)?;let mut s=self.sessions.get_mut(&r.session_id).ok_or_else(||Status::not_found("session not found"))?;if r.target_level<s.level{return Err(Status::invalid_argument("target level cannot be lower"));}
        self.tokens.remove(&s.token);s.level=r.target_level;s.scopes.extend(r.requested_scopes);s.token=Uuid::new_v4().to_string();s.expires_at=now_ms()+900_000;self.tokens.insert(s.token.clone(),s.id.clone());
        Ok(Response::new(ElevateSessionResponse{access_token:s.token.clone(),granted_level:s.level,granted_scopes:s.scopes.iter().cloned().collect(),expires_at_unix_ms:s.expires_at}))
    }
    async fn revoke_session(&self,request:Request<RevokeSessionRequest>)->Result<Response<RevokeSessionResponse>,Status>{let r=request.into_inner();let mut s=self.sessions.get_mut(&r.session_id).ok_or_else(||Status::not_found("session not found"))?;s.revoked=true;self.tokens.remove(&s.token);Ok(Response::new(RevokeSessionResponse{revoked:true}))}
}

fn invalid_session(reason:&str)->ValidateSessionResponse{ValidateSessionResponse{valid:false,session_id:String::new(),account_id:String::new(),granted_level:0,granted_scopes:Vec::new(),failure_reason:reason.into()}}
fn validate_level(level:i32)->Result<(),Status>{AccessLevel::try_from(level).ok().filter(|v|*v!=AccessLevel::Unspecified).map(|_|()).ok_or_else(||Status::invalid_argument("access level is required"))}
fn validate_scopes(level:i32,scopes:&[String])->Result<(),Status>{let allowed=match AccessLevel::try_from(level).unwrap_or(AccessLevel::Unspecified){AccessLevel::L0CorePlatform=>&["identifier.","profile."][..],AccessLevel::L1CoreFeature=>&["identifier.","profile.","friend.","group.","community.","oauth.","security."][..],AccessLevel::L2BusinessApplication=>&["identifier.","profile.","friend.","group.","community.","oauth.","security.","app.","miniapp.","business."][..],AccessLevel::Unspecified=>&[][..]};if scopes.iter().all(|s|allowed.iter().any(|p|s.starts_with(p))){Ok(())}else{Err(Status::permission_denied("scope is not allowed at requested level"))}}

#[derive(Default)] pub struct ConsentManagementServiceImpl { records:Arc<DashMap<String,ConsentRecord>> }

#[tonic::async_trait]
impl ConsentManagementService for ConsentManagementServiceImpl {
    async fn grant_consent(&self,request:Request<GrantConsentRequest>)->Result<Response<GrantConsentResponse>,Status>{let r=request.into_inner();let context=r.context.ok_or_else(||Status::unauthenticated("request context is required"))?;required(&context.user_id,"user_id")?;required(&r.owner_id,"owner_id")?;required(&r.purpose,"purpose")?;required(&r.policy_version,"policy_version")?;validate_owner(r.owner_type)?;if r.data_categories.is_empty(){return Err(Status::invalid_argument("data_categories are required"));}let id=Uuid::new_v4().to_string();let record=ConsentRecord{consent_id:id.clone(),user_id:context.user_id,owner_type:r.owner_type,owner_id:r.owner_id,purpose:r.purpose,data_categories:r.data_categories,scopes:r.scopes,status:ConsentStatus::Granted as i32,policy_version:r.policy_version,granted_at_unix_ms:now_ms(),expires_at_unix_ms:r.expires_at_unix_ms,revoked_at_unix_ms:0};self.records.insert(id,record.clone());Ok(Response::new(GrantConsentResponse{consent:Some(record)}))}
    async fn revoke_consent(&self,request:Request<RevokeConsentRequest>)->Result<Response<RevokeConsentResponse>,Status>{let r=request.into_inner();let mut record=self.records.get_mut(&r.consent_id).ok_or_else(||Status::not_found("consent not found"))?;record.status=ConsentStatus::Revoked as i32;record.revoked_at_unix_ms=now_ms();Ok(Response::new(RevokeConsentResponse{consent:Some(record.value().clone())}))}
    async fn evaluate_consent(&self,request:Request<EvaluateConsentRequest>)->Result<Response<EvaluateConsentResponse>,Status>{let r=request.into_inner();validate_owner(r.owner_type)?;let now=now_ms();let found=self.records.iter().find(|v|v.user_id==r.user_id&&v.owner_type==r.owner_type&&v.owner_id==r.owner_id&&v.purpose==r.purpose&&v.status==ConsentStatus::Granted as i32&&(v.expires_at_unix_ms==0||v.expires_at_unix_ms>now));let Some(record)=found else{return Ok(Response::new(EvaluateConsentResponse{allowed:false,consent_id:String::new(),missing_data_categories:r.required_data_categories,missing_scopes:r.required_scopes,failure_reason:"no active consent".into()}));};let categories=record.data_categories.iter().cloned().collect::<HashSet<_>>();let scopes=record.scopes.iter().cloned().collect::<HashSet<_>>();let missing_data_categories=r.required_data_categories.into_iter().filter(|v|!categories.contains(v)).collect::<Vec<_>>();let missing_scopes=r.required_scopes.into_iter().filter(|v|!scopes.contains(v)).collect::<Vec<_>>();let allowed=missing_data_categories.is_empty()&&missing_scopes.is_empty();Ok(Response::new(EvaluateConsentResponse{allowed,consent_id:record.consent_id.clone(),missing_data_categories,missing_scopes,failure_reason:if allowed{String::new()}else{"consent does not cover requested data or scope".into()}}))}
    async fn list_consents(&self,request:Request<ListConsentsRequest>)->Result<Response<ListConsentsResponse>,Status>{let r=request.into_inner();let consents=self.records.iter().filter(|v|v.user_id==r.user_id&&(r.owner_type==ConsentOwnerType::Unspecified as i32||v.owner_type==r.owner_type)).map(|v|v.value().clone()).collect();Ok(Response::new(ListConsentsResponse{consents}))}
}

fn validate_owner(owner:i32)->Result<(),Status>{ConsentOwnerType::try_from(owner).ok().filter(|v|*v!=ConsentOwnerType::Unspecified).map(|_|()).ok_or_else(||Status::invalid_argument("owner_type is required"))}

pub async fn shutdown(){let _=tokio::signal::ctrl_c().await;}
pub fn init_tracing(){let _=tracing_subscriber::fmt().json().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init();}

#[cfg(test)] mod tests {
    use super::*;
    use tektalk_contracts::v1::{MediaReference, RequestContext};

    fn context(user_id: &str) -> Option<RequestContext> {
        Some(RequestContext { user_id: user_id.into(), ..Default::default() })
    }

    fn text_message(user_id: &str, client_id: &str, text: &str) -> SendMessageRequest {
        SendMessageRequest {
            context: context(user_id),
            conversation_id: "conversation-1".into(),
            client_message_id: client_id.into(),
            kind: MessageKind::Text as i32,
            text: text.into(),
            ..Default::default()
        }
    }

    #[test] fn l0_rejects_business_scope(){assert!(validate_scopes(AccessLevel::L0CorePlatform as i32,&["app.read".into()]).is_err());}
    #[test] fn l2_accepts_business_scope(){assert!(validate_scopes(AccessLevel::L2BusinessApplication as i32,&["miniapp.launch".into()]).is_ok());}
    #[test] fn media_message_requires_reference(){let r=SendMessageRequest{kind:MessageKind::Photo as i32,..Default::default()};assert!(validate_message(&r).is_err());}

    #[test]
    fn every_supported_message_kind_enforces_its_payload_contract() {
        let text = SendMessageRequest { kind: MessageKind::Text as i32, text: "  ".into(), ..Default::default() };
        let sticker = SendMessageRequest { kind: MessageKind::Sticker as i32, ..Default::default() };
        let voice = SendMessageRequest { kind: MessageKind::Voice as i32, media: Some(MediaReference::default()), ..Default::default() };
        let unspecified = SendMessageRequest::default();
        assert!(validate_message(&text).is_err());
        assert!(validate_message(&sticker).is_err());
        assert!(validate_message(&voice).is_ok());
        assert!(validate_message(&unspecified).is_err());
    }

    #[tokio::test]
    async fn account_signup_login_covers_duplicate_password_and_device_trust() {
        let service = AccountServiceImpl::default();
        let signup = SignUpRequest {
            identifier: "+84901234567".into(), password: "Password123".into(),
            display_name: "TEK User".into(), device_name: "phone".into(), ..Default::default()
        };
        let created = service.sign_up(Request::new(signup.clone())).await.unwrap().into_inner();
        assert!(!created.account_id.is_empty());
        assert_eq!(service.sign_up(Request::new(signup)).await.unwrap_err().code(), tonic::Code::AlreadyExists);

        let wrong = SignInRequest { identifier: "+84901234567".into(), password: "wrong".into(), device_id: created.device_id.clone(), ..Default::default() };
        assert_eq!(service.sign_in(Request::new(wrong)).await.unwrap_err().code(), tonic::Code::Unauthenticated);

        let trusted = SignInRequest { identifier: "+84901234567".into(), password: "Password123".into(), device_id: created.device_id, ..Default::default() };
        let trusted = service.sign_in(Request::new(trusted)).await.unwrap().into_inner();
        assert!(!trusted.device_verification_required);
        assert!(!trusted.access_token.is_empty());

        let unknown = SignInRequest { identifier: "+84901234567".into(), password: "Password123".into(), device_id: "new-device".into(), ..Default::default() };
        let unknown = service.sign_in(Request::new(unknown)).await.unwrap().into_inner();
        assert!(unknown.device_verification_required);
        assert!(unknown.access_token.is_empty());
        assert!(!unknown.challenge_id.is_empty());
    }

    #[tokio::test]
    async fn chat_is_idempotent_and_paginates_in_chronological_order() {
        let service = ChatServiceImpl::default();
        let first = service.send_message(Request::new(text_message("u1", "c1", "one"))).await.unwrap().into_inner();
        let duplicate = service.send_message(Request::new(text_message("u1", "c1", "changed"))).await.unwrap().into_inner();
        assert_eq!(duplicate.server_message_id, first.server_message_id);
        assert!(duplicate.deduplicated);

        let second = service.send_message(Request::new(text_message("u1", "c2", "two"))).await.unwrap().into_inner();
        let third = service.send_message(Request::new(text_message("u1", "c3", "three"))).await.unwrap().into_inner();
        let page = service.get_messages(Request::new(GetMessagesRequest {
            conversation_id: "conversation-1".into(), limit: 2, ..Default::default()
        })).await.unwrap().into_inner();
        assert_eq!(page.messages.iter().map(|m| m.text.as_str()).collect::<Vec<_>>(), ["two", "three"]);
        assert_eq!(page.next_before_message_id, second.server_message_id);

        let previous = service.get_messages(Request::new(GetMessagesRequest {
            conversation_id: "conversation-1".into(), before_message_id: page.next_before_message_id,
            limit: 1000, ..Default::default()
        })).await.unwrap().into_inner();
        assert_eq!(previous.messages.len(), 1);
        assert_eq!(previous.messages[0].server_message_id, first.server_message_id);
        assert!(third.server_message_id > second.server_message_id);
    }

    #[tokio::test]
    async fn chat_deduplication_is_scoped_to_sender() {
        let service = ChatServiceImpl::default();
        let first = service.send_message(Request::new(text_message("u1", "same-id", "one"))).await.unwrap().into_inner();
        let second = service.send_message(Request::new(text_message("u2", "same-id", "two"))).await.unwrap().into_inner();
        assert!(!first.deduplicated && !second.deduplicated);
        assert_ne!(first.server_message_id, second.server_message_id);
    }

    async fn l0_session(service: &SessionManagementServiceImpl) -> CreateSessionResponse {
        service.create_session(Request::new(CreateSessionRequest {
            account_id: "account-1".into(), device_id: "device-1".into(),
            requested_level: AccessLevel::L0CorePlatform as i32, audience: "core".into(),
            requested_scopes: vec!["profile.read".into()], ..Default::default()
        })).await.unwrap().into_inner()
    }

    #[tokio::test]
    async fn session_validation_checks_scope_audience_level_and_revocation() {
        let service = SessionManagementServiceImpl::default();
        let created = l0_session(&service).await;
        let valid = service.validate_session(Request::new(ValidateSessionRequest {
            access_token: created.access_token.clone(), required_level: AccessLevel::L0CorePlatform as i32,
            required_scopes: vec!["profile.read".into()], audience: "core".into(), ..Default::default()
        })).await.unwrap().into_inner();
        assert!(valid.valid);

        for request in [
            ValidateSessionRequest { access_token: created.access_token.clone(), required_level: AccessLevel::L1CoreFeature as i32, audience: "core".into(), ..Default::default() },
            ValidateSessionRequest { access_token: created.access_token.clone(), required_level: AccessLevel::L0CorePlatform as i32, required_scopes: vec!["profile.write".into()], audience: "core".into(), ..Default::default() },
            ValidateSessionRequest { access_token: created.access_token.clone(), required_level: AccessLevel::L0CorePlatform as i32, audience: "other".into(), ..Default::default() },
        ] { assert!(!service.validate_session(Request::new(request)).await.unwrap().into_inner().valid); }

        service.revoke_session(Request::new(RevokeSessionRequest { session_id: created.session_id, reason: "logout".into(), ..Default::default() })).await.unwrap();
        let revoked = service.validate_session(Request::new(ValidateSessionRequest { access_token: created.access_token, audience: "core".into(), ..Default::default() })).await.unwrap().into_inner();
        assert!(!revoked.valid);
        assert_eq!(revoked.failure_reason, "unknown token");
    }

    #[tokio::test]
    async fn session_elevation_requires_proof_and_rotates_token() {
        let service = SessionManagementServiceImpl::default();
        let created = l0_session(&service).await;
        let no_proof = ElevateSessionRequest { session_id: created.session_id.clone(), target_level: AccessLevel::L1CoreFeature as i32, requested_scopes: vec!["friend.read".into()], ..Default::default() };
        assert_eq!(service.elevate_session(Request::new(no_proof)).await.unwrap_err().code(), tonic::Code::InvalidArgument);
        let elevated = service.elevate_session(Request::new(ElevateSessionRequest {
            session_id: created.session_id, target_level: AccessLevel::L1CoreFeature as i32,
            requested_scopes: vec!["friend.read".into()], security_proof: "step-up-proof".into(), ..Default::default()
        })).await.unwrap().into_inner();
        assert_ne!(elevated.access_token, created.access_token);
        assert!(elevated.granted_scopes.contains(&"profile.read".into()));
        assert!(elevated.granted_scopes.contains(&"friend.read".into()));
        let old = service.validate_session(Request::new(ValidateSessionRequest { access_token: created.access_token, audience: "core".into(), ..Default::default() })).await.unwrap().into_inner();
        assert!(!old.valid);
    }

    fn grant(user: &str, expires_at: i64) -> GrantConsentRequest {
        GrantConsentRequest { context: context(user), owner_type: ConsentOwnerType::ThirdParty as i32,
            owner_id: "partner-1".into(), purpose: "personalization".into(), data_categories: vec!["profile".into()],
            scopes: vec!["profile.read".into()], policy_version: "v1".into(), expires_at_unix_ms: expires_at }
    }

    fn evaluate(user: &str) -> EvaluateConsentRequest {
        EvaluateConsentRequest { user_id: user.into(), owner_type: ConsentOwnerType::ThirdParty as i32,
            owner_id: "partner-1".into(), purpose: "personalization".into(),
            required_data_categories: vec!["profile".into()], required_scopes: vec!["profile.read".into()], ..Default::default() }
    }

    #[tokio::test]
    async fn consent_lifecycle_covers_grant_missing_scope_expiry_and_revoke() {
        let service = ConsentManagementServiceImpl::default();
        let granted = service.grant_consent(Request::new(grant("u1", 0))).await.unwrap().into_inner().consent.unwrap();
        assert!(service.evaluate_consent(Request::new(evaluate("u1"))).await.unwrap().into_inner().allowed);

        let mut missing = evaluate("u1"); missing.required_scopes.push("contacts.read".into());
        let missing = service.evaluate_consent(Request::new(missing)).await.unwrap().into_inner();
        assert!(!missing.allowed);
        assert_eq!(missing.missing_scopes, ["contacts.read"]);

        service.grant_consent(Request::new(grant("expired-user", now_ms() - 1))).await.unwrap();
        let expired = service.evaluate_consent(Request::new(evaluate("expired-user"))).await.unwrap().into_inner();
        assert!(!expired.allowed);
        assert_eq!(expired.failure_reason, "no active consent");

        service.revoke_consent(Request::new(RevokeConsentRequest { consent_id: granted.consent_id, reason: "user request".into(), ..Default::default() })).await.unwrap();
        assert!(!service.evaluate_consent(Request::new(evaluate("u1"))).await.unwrap().into_inner().allowed);
    }

    #[tokio::test]
    async fn consent_listing_filters_owner_type_and_user() {
        let service = ConsentManagementServiceImpl::default();
        service.grant_consent(Request::new(grant("u1", 0))).await.unwrap();
        let mut platform = grant("u1", 0); platform.owner_type = ConsentOwnerType::Platform as i32; platform.owner_id = "tektalk".into();
        service.grant_consent(Request::new(platform)).await.unwrap();
        service.grant_consent(Request::new(grant("u2", 0))).await.unwrap();
        let listed = service.list_consents(Request::new(ListConsentsRequest { user_id: "u1".into(), owner_type: ConsentOwnerType::Platform as i32, ..Default::default() })).await.unwrap().into_inner();
        assert_eq!(listed.consents.len(), 1);
        assert_eq!(listed.consents[0].owner_id, "tektalk");
    }
}
