use async_trait::async_trait;
use dashmap::DashMap;
use sqlx::PgPool;
use std::sync::Arc;
use tektalk_contracts::v1::{product_catalog_service_server::ProductCatalogService,subscription_service_server::SubscriptionService,ActivateRequest,ActivateResponse,CancelRequest,CancelResponse,CustomerSegment,GetEntitlementsRequest,GetEntitlementsResponse,ListPlansRequest,ListPlansResponse,ProductPlan,ResolveEntitlementsRequest,ResolveEntitlementsResponse,Subscription,SubscriptionStatus};
use tektalk_product_catalog::{reference_catalog,Catalog,Segment};
use tonic::{Request,Response,Status};

#[async_trait]
trait SubscriptionRepository:Send+Sync{
 async fn activate(&self,subject:&str,segment:i32,plan:&str)->anyhow::Result<Subscription>;
 async fn cancel(&self,subject:&str,segment:i32)->anyhow::Result<Option<Subscription>>;
 async fn get(&self,subject:&str,segment:i32)->anyhow::Result<Option<Subscription>>;
}

#[derive(Default)] struct MemorySubscriptions{values:DashMap<String,Subscription>}
fn key(subject:&str,segment:i32)->String{format!("{segment}:{subject}")}

#[async_trait]
impl SubscriptionRepository for MemorySubscriptions{
 async fn activate(&self,subject:&str,segment:i32,plan:&str)->anyhow::Result<Subscription>{let value=Subscription{id:uuid::Uuid::new_v4().to_string(),subject_id:subject.into(),subject_segment:segment,plan_code:plan.into(),status:SubscriptionStatus::Active as i32};self.values.insert(key(subject,segment),value.clone());Ok(value)}
 async fn cancel(&self,subject:&str,segment:i32)->anyhow::Result<Option<Subscription>>{let Some(mut value)=self.values.get_mut(&key(subject,segment))else{return Ok(None)};value.status=SubscriptionStatus::Cancelled as i32;Ok(Some(value.clone()))}
 async fn get(&self,subject:&str,segment:i32)->anyhow::Result<Option<Subscription>>{Ok(self.values.get(&key(subject,segment)).map(|value|value.clone()))}
}

struct PostgresSubscriptions{pool:PgPool}
#[derive(sqlx::FromRow)] struct SubscriptionRow{id:String,subject_id:String,subject_segment:i32,plan_code:String,status:String}
impl From<SubscriptionRow> for Subscription{fn from(value:SubscriptionRow)->Self{Self{id:value.id,subject_id:value.subject_id,subject_segment:value.subject_segment,plan_code:value.plan_code,status:if value.status=="active"{SubscriptionStatus::Active as i32}else{SubscriptionStatus::Cancelled as i32}}}}

#[async_trait]
impl SubscriptionRepository for PostgresSubscriptions{
 async fn activate(&self,subject:&str,segment:i32,plan:&str)->anyhow::Result<Subscription>{let mut tx=self.pool.begin().await?;let row=sqlx::query_as::<_,SubscriptionRow>("INSERT INTO commerce_subscriptions(id,subject_id,subject_segment,plan_code,status,created_at,updated_at) VALUES(gen_random_uuid(),$1,$2,$3,'active',now(),now()) ON CONFLICT(subject_id,subject_segment) DO UPDATE SET plan_code=EXCLUDED.plan_code,status='active',updated_at=now() RETURNING id::text,subject_id,subject_segment,plan_code,status").bind(subject).bind(segment).bind(plan).fetch_one(&mut *tx).await?;sqlx::query("INSERT INTO commerce_outbox(id,event_type,aggregate_id,payload,created_at) VALUES(gen_random_uuid(),'subscription.activated',$1,jsonb_build_object('subject_id',$2,'segment',$3,'plan_code',$4),now())").bind(&row.id).bind(subject).bind(segment).bind(plan).execute(&mut *tx).await?;tx.commit().await?;Ok(row.into())}
 async fn cancel(&self,subject:&str,segment:i32)->anyhow::Result<Option<Subscription>>{let mut tx=self.pool.begin().await?;let row=sqlx::query_as::<_,SubscriptionRow>("UPDATE commerce_subscriptions SET status='cancelled',updated_at=now() WHERE subject_id=$1 AND subject_segment=$2 RETURNING id::text,subject_id,subject_segment,plan_code,status").bind(subject).bind(segment).fetch_optional(&mut *tx).await?;if let Some(value)=&row{sqlx::query("INSERT INTO commerce_outbox(id,event_type,aggregate_id,payload,created_at) VALUES(gen_random_uuid(),'subscription.cancelled',$1,jsonb_build_object('subject_id',$2,'segment',$3),now())").bind(&value.id).bind(subject).bind(segment).execute(&mut *tx).await?;}tx.commit().await?;Ok(row.map(Into::into))}
 async fn get(&self,subject:&str,segment:i32)->anyhow::Result<Option<Subscription>>{Ok(sqlx::query_as::<_,SubscriptionRow>("SELECT id::text,subject_id,subject_segment,plan_code,status FROM commerce_subscriptions WHERE subject_id=$1 AND subject_segment=$2").bind(subject).bind(segment).fetch_optional(&self.pool).await?.map(Into::into))}
}

#[derive(Clone)] pub struct CommerceService{catalog:Arc<Catalog>,subscriptions:Arc<dyn SubscriptionRepository>}
impl Default for CommerceService{fn default()->Self{Self{catalog:Arc::new(reference_catalog()),subscriptions:Arc::new(MemorySubscriptions::default())}}}
impl CommerceService{pub fn with_postgres(pool:PgPool)->Self{Self{catalog:Arc::new(reference_catalog()),subscriptions:Arc::new(PostgresSubscriptions{pool})}}}
fn segment(value:i32)->Result<Segment,Status>{match CustomerSegment::try_from(value){Ok(CustomerSegment::User)=>Ok(Segment::User),Ok(CustomerSegment::Enterprise)=>Ok(Segment::Enterprise),_=>Err(Status::invalid_argument("customer segment is required"))}}
fn proto_segment(value:Segment)->i32{match value{Segment::User=>CustomerSegment::User as i32,Segment::Enterprise=>CustomerSegment::Enterprise as i32}}
fn storage_error(error:anyhow::Error)->Status{tracing::error!(?error,"subscription storage failed");Status::internal("subscription storage unavailable")}

#[tonic::async_trait]
impl ProductCatalogService for CommerceService{
 async fn list_plans(&self,request:Request<ListPlansRequest>)->Result<Response<ListPlansResponse>,Status>{let requested=segment(request.into_inner().segment)?;let plans=self.catalog.list(requested).into_iter().map(|p|ProductPlan{code:p.code.clone(),segment:proto_segment(p.segment),display_name:p.display_name.clone(),currency:p.price.currency.clone(),price_minor_units:p.price.minor_units,entitlement_keys:p.entitlements.iter().cloned().collect(),active:p.active}).collect();Ok(Response::new(ListPlansResponse{plans}))}
 async fn resolve_entitlements(&self,request:Request<ResolveEntitlementsRequest>)->Result<Response<ResolveEntitlementsResponse>,Status>{let input=request.into_inner();let values=self.catalog.resolve_entitlements(&input.plan_code,segment(input.subject_segment)?).map_err(|e|Status::failed_precondition(format!("{e:?}")))?;Ok(Response::new(ResolveEntitlementsResponse{entitlement_keys:values.into_iter().collect()}))}
}

#[tonic::async_trait]
impl SubscriptionService for CommerceService{
 async fn activate(&self,request:Request<ActivateRequest>)->Result<Response<ActivateResponse>,Status>{let input=request.into_inner();if input.subject_id.trim().is_empty(){return Err(Status::invalid_argument("subject_id is required"));}let kind=segment(input.subject_segment)?;self.catalog.resolve_entitlements(&input.plan_code,kind).map_err(|e|Status::failed_precondition(format!("{e:?}")))?;let value=self.subscriptions.activate(&input.subject_id,input.subject_segment,&input.plan_code).await.map_err(storage_error)?;Ok(Response::new(ActivateResponse{subscription:Some(value)}))}
 async fn cancel(&self,request:Request<CancelRequest>)->Result<Response<CancelResponse>,Status>{let input=request.into_inner();segment(input.subject_segment)?;let value=self.subscriptions.cancel(&input.subject_id,input.subject_segment).await.map_err(storage_error)?.ok_or_else(||Status::not_found("subscription not found"))?;Ok(Response::new(CancelResponse{subscription:Some(value)}))}
 async fn get_entitlements(&self,request:Request<GetEntitlementsRequest>)->Result<Response<GetEntitlementsResponse>,Status>{let input=request.into_inner();let kind=segment(input.subject_segment)?;let value=self.subscriptions.get(&input.subject_id,input.subject_segment).await.map_err(storage_error)?.ok_or_else(||Status::not_found("subscription not found"))?;if value.status!=SubscriptionStatus::Active as i32{return Err(Status::failed_precondition("subscription is not active"));}let values=self.catalog.resolve_entitlements(&value.plan_code,kind).map_err(|e|Status::failed_precondition(format!("{e:?}")))?;Ok(Response::new(GetEntitlementsResponse{entitlement_keys:values.into_iter().collect()}))}
}

#[cfg(test)] mod tests{use super::*;#[tokio::test]async fn lifecycle_revokes_entitlements(){let service=CommerceService::default();service.activate(Request::new(ActivateRequest{subject_id:"user-1".into(),subject_segment:CustomerSegment::User as i32,plan_code:"user.premium".into()})).await.unwrap();let query=GetEntitlementsRequest{subject_id:"user-1".into(),subject_segment:CustomerSegment::User as i32};assert!(service.get_entitlements(Request::new(query.clone())).await.unwrap().into_inner().entitlement_keys.contains(&"ai.assistant.standard".into()));service.cancel(Request::new(CancelRequest{subject_id:"user-1".into(),subject_segment:CustomerSegment::User as i32})).await.unwrap();assert_eq!(service.get_entitlements(Request::new(query)).await.unwrap_err().code(),tonic::Code::FailedPrecondition);}#[tokio::test]async fn segment_is_enforced(){let service=CommerceService::default();let result=service.activate(Request::new(ActivateRequest{subject_id:"user-1".into(),subject_segment:CustomerSegment::User as i32,plan_code:"enterprise.oa-business".into()})).await;assert_eq!(result.unwrap_err().code(),tonic::Code::FailedPrecondition);}}

