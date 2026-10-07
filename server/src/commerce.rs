use axum::{extract::{Query, State}, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};
use tektalk_contracts::v1::{
    product_catalog_service_client::ProductCatalogServiceClient,
    subscription_service_client::SubscriptionServiceClient,
    ActivateRequest, CancelRequest, CustomerSegment, GetEntitlementsRequest,
    ListPlansRequest,
};

use crate::{auth, error::{AppError, AppResult}, state::AppState};

#[derive(Deserialize)]
pub struct CatalogQuery { pub segment: Option<String> }

#[derive(Deserialize)]
pub struct ActivateBody { pub plan_code: String }

#[derive(Serialize)]
pub struct PlanView {
    pub code: String,
    pub segment: String,
    pub display_name: String,
    pub currency: String,
    pub price_minor_units: u64,
    pub entitlement_keys: Vec<String>,
}

#[derive(Serialize)]
pub struct SubscriptionView {
    pub id: String,
    pub plan_code: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct EntitlementsView { pub entitlement_keys: Vec<String> }

fn user_segment() -> i32 { CustomerSegment::User as i32 }

fn grpc_error(error: tonic::Status) -> AppError {
    tracing::warn!(code=?error.code(), "business service request failed");
    match error.code() {
        tonic::Code::InvalidArgument => AppError::Invalid("invalid commerce request"),
        tonic::Code::FailedPrecondition => AppError::Conflict("commercial condition not met"),
        tonic::Code::NotFound => AppError::Invalid("subscription not found"),
        _ => AppError::Internal(anyhow::anyhow!("business service unavailable")),
    }
}

pub async fn list_plans(
    State(state): State<AppState>,
    Query(query): Query<CatalogQuery>,
) -> AppResult<Json<Vec<PlanView>>> {
    let segment = match query.segment.as_deref().unwrap_or("user") {
        "user" => CustomerSegment::User,
        "enterprise" => CustomerSegment::Enterprise,
        _ => return Err(AppError::Invalid("segment must be user or enterprise")),
    };
    let mut client = ProductCatalogServiceClient::new(state.business_channel.clone());
    let response = client.list_plans(ListPlansRequest { segment: segment as i32 }).await.map_err(grpc_error)?.into_inner();
    Ok(Json(response.plans.into_iter().map(|plan| PlanView {
        code: plan.code,
        segment: if plan.segment == CustomerSegment::Enterprise as i32 { "enterprise" } else { "user" }.to_owned(),
        display_name: plan.display_name,
        currency: plan.currency,
        price_minor_units: plan.price_minor_units,
        entitlement_keys: plan.entitlement_keys,
    }).collect()))
}

pub async fn activate(
    State(state): State<AppState>, headers: HeaderMap, Json(body): Json<ActivateBody>,
) -> AppResult<Json<SubscriptionView>> {
    let claims = auth::claims(&state, &headers)?;
    let mut client = SubscriptionServiceClient::new(state.business_channel.clone());
    let response = client.activate(ActivateRequest {
        subject_id: claims.sub.to_string(), subject_segment: user_segment(), plan_code: body.plan_code,
    }).await.map_err(grpc_error)?.into_inner();
    let subscription = response.subscription.ok_or_else(|| AppError::Internal(anyhow::anyhow!("empty subscription response")))?;
    Ok(Json(SubscriptionView { id: subscription.id, plan_code: subscription.plan_code, status: "active".to_owned() }))
}

pub async fn cancel(
    State(state): State<AppState>, headers: HeaderMap,
) -> AppResult<Json<SubscriptionView>> {
    let claims = auth::claims(&state, &headers)?;
    let mut client = SubscriptionServiceClient::new(state.business_channel.clone());
    let response = client.cancel(CancelRequest {
        subject_id: claims.sub.to_string(), subject_segment: user_segment(),
    }).await.map_err(grpc_error)?.into_inner();
    let subscription = response.subscription.ok_or_else(|| AppError::Internal(anyhow::anyhow!("empty subscription response")))?;
    Ok(Json(SubscriptionView { id: subscription.id, plan_code: subscription.plan_code, status: "cancelled".to_owned() }))
}

pub async fn entitlements(
    State(state): State<AppState>, headers: HeaderMap,
) -> AppResult<Json<EntitlementsView>> {
    let claims = auth::claims(&state, &headers)?;
    let mut client = SubscriptionServiceClient::new(state.business_channel.clone());
    let response = client.get_entitlements(GetEntitlementsRequest {
        subject_id: claims.sub.to_string(), subject_segment: user_segment(),
    }).await.map_err(grpc_error)?.into_inner();
    Ok(Json(EntitlementsView { entitlement_keys: response.entitlement_keys }))
}

