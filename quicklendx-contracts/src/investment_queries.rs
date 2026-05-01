use crate::investment::InvestmentStorage;
use crate::types::InvestmentStatus;
use soroban_sdk::{Address, BytesN, Env, Vec};

/// Maximum number of records returned by paginated query endpoints.
pub const MAX_QUERY_LIMIT: u32 = 100;

/// Read-only investment query helpers with pagination support
pub struct InvestmentQueries;

impl InvestmentQueries {
    /// Caps query limit to prevent excessive memory usage.
    #[inline]
    pub fn cap_query_limit(limit: u32) -> u32 {
        limit.min(MAX_QUERY_LIMIT)
    }

    /// Safely calculates pagination bounds with overflow protection.
    fn calculate_safe_bounds(offset: u32, limit: u32, collection_size: u32) -> (u32, u32) {
        let capped_limit = Self::cap_query_limit(limit);
        let start = offset.min(collection_size);
        let end = start.saturating_add(capped_limit).min(collection_size);
        (start, end)
    }

    /// Retrieves paginated investments for a specific investor.
    pub fn get_investor_investments_paginated(
        env: &Env,
        investor: &Address,
        status_filter: Option<InvestmentStatus>,
        offset: u32,
        limit: u32,
    ) -> Vec<BytesN<32>> {
        let all_investment_ids = InvestmentStorage::get_investments_by_investor(env, investor);
        let mut filtered = Vec::new(env);

        for investment_id in all_investment_ids.iter() {
            if let Some(investment) = InvestmentStorage::get_investment(env, &investment_id) {
                let matches_filter = match &status_filter {
                    Some(status) => investment.status == *status,
                    None => true,
                };
                if matches_filter {
                    filtered.push_back(investment_id);
                }
            }
        }

        let collection_size = filtered.len() as u32;
        let (start, end) = Self::calculate_safe_bounds(offset, limit, collection_size);

        let mut result = Vec::new(env);
        let mut idx = start;
        while idx < end {
            if let Some(investment_id) = filtered.get(idx) {
                result.push_back(investment_id);
            }
            idx = idx.saturating_add(1);
        }
        result
    }
}
