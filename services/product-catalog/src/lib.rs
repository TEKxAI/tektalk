//! Business Solutions product catalog and entitlement rules.
//!
//! This crate deliberately has no infrastructure dependencies. Persistence,
//! billing providers and gRPC transport are adapters around this domain model.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Segment {
    User,
    Enterprise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingPeriod {
    Month,
    Year,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    pub currency: String,
    pub minor_units: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub code: String,
    pub segment: Segment,
    pub display_name: String,
    pub price: Money,
    pub period: BillingPeriod,
    /// Stable capability identifiers owned by Platform Services.
    pub entitlements: BTreeSet<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    EmptyCode,
    InvalidCurrency,
    MissingEntitlements,
    DuplicatePlan(String),
    UnknownPlan(String),
    SegmentMismatch,
    InactivePlan,
}

#[derive(Debug, Default)]
pub struct Catalog {
    plans: BTreeMap<String, Plan>,
}

impl Catalog {
    pub fn add(&mut self, plan: Plan) -> Result<(), CatalogError> {
        validate_plan(&plan)?;
        if self.plans.contains_key(&plan.code) {
            return Err(CatalogError::DuplicatePlan(plan.code));
        }
        self.plans.insert(plan.code.clone(), plan);
        Ok(())
    }

    pub fn list(&self, segment: Segment) -> Vec<&Plan> {
        self.plans
            .values()
            .filter(|plan| plan.segment == segment && plan.active)
            .collect()
    }

    pub fn resolve_entitlements(
        &self,
        plan_code: &str,
        subject_segment: Segment,
    ) -> Result<BTreeSet<String>, CatalogError> {
        let plan = self
            .plans
            .get(plan_code)
            .ok_or_else(|| CatalogError::UnknownPlan(plan_code.to_owned()))?;
        if !plan.active {
            return Err(CatalogError::InactivePlan);
        }
        if plan.segment != subject_segment {
            return Err(CatalogError::SegmentMismatch);
        }
        Ok(plan.entitlements.clone())
    }
}

pub fn validate_plan(plan: &Plan) -> Result<(), CatalogError> {
    if plan.code.trim().is_empty() {
        return Err(CatalogError::EmptyCode);
    }
    if plan.price.currency.len() != 3
        || !plan.price.currency.chars().all(|c| c.is_ascii_uppercase())
    {
        return Err(CatalogError::InvalidCurrency);
    }
    if plan.entitlements.is_empty() {
        return Err(CatalogError::MissingEntitlements);
    }
    Ok(())
}

pub fn reference_catalog() -> Catalog {
    let mut catalog = Catalog::default();
    for plan in [
        plan("user.premium", Segment::User, "Premium", 79_000, &["message.history.extended", "ai.assistant.standard", "storage.100gb"]),
        plan("user.professional", Segment::User, "Professional", 149_000, &["message.history.extended", "ai.assistant.pro", "channel.creator", "storage.500gb"]),
        plan("enterprise.oa-business", Segment::Enterprise, "OA Business", 499_000, &["oa.business", "bot.standard", "analytics.standard"]),
        plan("enterprise.miniapp-business", Segment::Enterprise, "Mini-App Business", 999_000, &["miniapp.publish", "miniapp.commerce", "openapi.standard", "analytics.standard"]),
    ] {
        catalog.add(plan).expect("reference plan must be valid");
    }
    catalog
}

fn plan(code: &str, segment: Segment, name: &str, price: u64, entitlements: &[&str]) -> Plan {
    Plan {
        code: code.to_owned(),
        segment,
        display_name: name.to_owned(),
        price: Money { currency: "VND".to_owned(), minor_units: price },
        period: BillingPeriod::Month,
        entitlements: entitlements.iter().map(|value| (*value).to_owned()).collect(),
        active: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_catalog_separates_uvas_and_evas() {
        let catalog = reference_catalog();
        assert_eq!(catalog.list(Segment::User).len(), 2);
        assert_eq!(catalog.list(Segment::Enterprise).len(), 2);
    }

    #[test]
    fn enterprise_plan_cannot_be_assigned_to_user() {
        let catalog = reference_catalog();
        assert_eq!(
            catalog.resolve_entitlements("enterprise.oa-business", Segment::User),
            Err(CatalogError::SegmentMismatch)
        );
    }

    #[test]
    fn inactive_plan_is_not_sellable() {
        let mut catalog = Catalog::default();
        let mut plan = super::plan("user.retired", Segment::User, "Retired", 1, &["message.basic"]);
        plan.active = false;
        catalog.add(plan).unwrap();
        assert!(catalog.list(Segment::User).is_empty());
        assert_eq!(catalog.resolve_entitlements("user.retired", Segment::User), Err(CatalogError::InactivePlan));
    }

    #[test]
    fn rejects_duplicate_and_invalid_plans() {
        let mut catalog = Catalog::default();
        let valid = super::plan("user.premium", Segment::User, "Premium", 1, &["message.basic"]);
        catalog.add(valid.clone()).unwrap();
        assert_eq!(catalog.add(valid), Err(CatalogError::DuplicatePlan("user.premium".to_owned())));

        let mut invalid = super::plan("bad", Segment::User, "Bad", 1, &["message.basic"]);
        invalid.price.currency = "vnd".to_owned();
        assert_eq!(catalog.add(invalid), Err(CatalogError::InvalidCurrency));
    }
}

