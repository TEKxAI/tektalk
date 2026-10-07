use dashmap::DashMap;
use std::sync::Arc;
use tektalk_contracts::v1::{
    product_catalog_service_server::ProductCatalogService,
    subscription_service_server::SubscriptionService,
    ActivateSubscriptionRequest, CancelSubscriptionRequest, CustomerSegment,
    GetEntitlementsRequest, ListPlansRequest, ListPlansResponse, ProductPlan,
    ResolveEntitlementsRequest, ResolveEntitlementsResponse, Subscription,
    SubscriptionResponse, SubscriptionStatus,
};
use tektalk_product_catalog::{reference_catalog, Catalog, Segment};
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct CommerceService {
    catalog: Arc<Catalog>,
    subscriptions: Arc<DashMap<String, Subscription>>,
}

impl Default for CommerceService {
    fn default() -> Self {
        Self {
            catalog: Arc::new(reference_catalog()),
            subscriptions: Arc::new(DashMap::new()),
        }
    }
}

fn domain_segment(value: i32) -> Result<Segment, Status> {
    match CustomerSegment::try_from(value) {
        Ok(CustomerSegment::User) => Ok(Segment::User),
        Ok(CustomerSegment::Enterprise) => Ok(Segment::Enterprise),
        _ => Err(Status::invalid_argument("customer segment is required")),
    }
}

fn proto_segment(value: Segment) -> i32 {
    match value {
        Segment::User => CustomerSegment::User as i32,
        Segment::Enterprise => CustomerSegment::Enterprise as i32,
    }
}

fn subscription_key(subject_id: &str, segment: Segment) -> String {
    format!("{:?}:{subject_id}", segment)
}

#[tonic::async_trait]
impl ProductCatalogService for CommerceService {
    async fn list_plans(
        &self,
        request: Request<ListPlansRequest>,
    ) -> Result<Response<ListPlansResponse>, Status> {
        let segment = domain_segment(request.into_inner().segment)?;
        let plans = self
            .catalog
            .list(segment)
            .into_iter()
            .map(|plan| ProductPlan {
                code: plan.code.clone(),
                segment: proto_segment(plan.segment),
                display_name: plan.display_name.clone(),
                currency: plan.price.currency.clone(),
                price_minor_units: plan.price.minor_units,
                entitlement_keys: plan.entitlements.iter().cloned().collect(),
                active: plan.active,
            })
            .collect();
        Ok(Response::new(ListPlansResponse { plans }))
    }

    async fn resolve_entitlements(
        &self,
        request: Request<ResolveEntitlementsRequest>,
    ) -> Result<Response<ResolveEntitlementsResponse>, Status> {
        let input = request.into_inner();
        let segment = domain_segment(input.subject_segment)?;
        let entitlements = self
            .catalog
            .resolve_entitlements(&input.plan_code, segment)
            .map_err(|error| Status::failed_precondition(format!("{error:?}")))?;
        Ok(Response::new(ResolveEntitlementsResponse {
            entitlement_keys: entitlements.into_iter().collect(),
        }))
    }
}

#[tonic::async_trait]
impl SubscriptionService for CommerceService {
    async fn activate(
        &self,
        request: Request<ActivateSubscriptionRequest>,
    ) -> Result<Response<SubscriptionResponse>, Status> {
        let input = request.into_inner();
        if input.subject_id.trim().is_empty() {
            return Err(Status::invalid_argument("subject_id is required"));
        }
        let segment = domain_segment(input.subject_segment)?;
        self.catalog
            .resolve_entitlements(&input.plan_code, segment)
            .map_err(|error| Status::failed_precondition(format!("{error:?}")))?;
        let subscription = Subscription {
            id: uuid::Uuid::new_v4().to_string(),
            subject_id: input.subject_id.clone(),
            subject_segment: input.subject_segment,
            plan_code: input.plan_code,
            status: SubscriptionStatus::Active as i32,
        };
        self.subscriptions.insert(
            subscription_key(&input.subject_id, segment),
            subscription.clone(),
        );
        Ok(Response::new(SubscriptionResponse {
            subscription: Some(subscription),
        }))
    }

    async fn cancel(
        &self,
        request: Request<CancelSubscriptionRequest>,
    ) -> Result<Response<SubscriptionResponse>, Status> {
        let input = request.into_inner();
        let segment = domain_segment(input.subject_segment)?;
        let key = subscription_key(&input.subject_id, segment);
        let mut subscription = self
            .subscriptions
            .get_mut(&key)
            .ok_or_else(|| Status::not_found("active subscription not found"))?;
        subscription.status = SubscriptionStatus::Cancelled as i32;
        Ok(Response::new(SubscriptionResponse {
            subscription: Some(subscription.clone()),
        }))
    }

    async fn get_entitlements(
        &self,
        request: Request<GetEntitlementsRequest>,
    ) -> Result<Response<ResolveEntitlementsResponse>, Status> {
        let input = request.into_inner();
        let segment = domain_segment(input.subject_segment)?;
        let subscription = self
            .subscriptions
            .get(&subscription_key(&input.subject_id, segment))
            .ok_or_else(|| Status::not_found("active subscription not found"))?;
        if subscription.status != SubscriptionStatus::Active as i32 {
            return Err(Status::failed_precondition("subscription is not active"));
        }
        let entitlements = self
            .catalog
            .resolve_entitlements(&subscription.plan_code, segment)
            .map_err(|error| Status::failed_precondition(format!("{error:?}")))?;
        Ok(Response::new(ResolveEntitlementsResponse {
            entitlement_keys: entitlements.into_iter().collect(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn activation_unlocks_entitlements_and_cancel_revokes_them() {
        let service = CommerceService::default();
        let activate = ActivateSubscriptionRequest {
            subject_id: "user-1".into(),
            subject_segment: CustomerSegment::User as i32,
            plan_code: "user.premium".into(),
        };
        service.activate(Request::new(activate)).await.unwrap();
        let query = GetEntitlementsRequest {
            subject_id: "user-1".into(),
            subject_segment: CustomerSegment::User as i32,
        };
        let result = service.get_entitlements(Request::new(query.clone())).await.unwrap();
        assert!(result.into_inner().entitlement_keys.contains(&"ai.assistant.standard".to_owned()));
        service.cancel(Request::new(CancelSubscriptionRequest {
            subject_id: "user-1".into(),
            subject_segment: CustomerSegment::User as i32,
        })).await.unwrap();
        assert_eq!(service.get_entitlements(Request::new(query)).await.unwrap_err().code(), tonic::Code::FailedPrecondition);
    }

    #[tokio::test]
    async fn enterprise_plan_cannot_be_activated_for_user() {
        let service = CommerceService::default();
        let result = service.activate(Request::new(ActivateSubscriptionRequest {
            subject_id: "user-1".into(),
            subject_segment: CustomerSegment::User as i32,
            plan_code: "enterprise.oa-business".into(),
        })).await;
        assert_eq!(result.unwrap_err().code(), tonic::Code::FailedPrecondition);
    }
}

