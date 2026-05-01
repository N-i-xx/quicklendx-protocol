#![allow(dead_code)]
use crate::admin::AdminStorage;
use crate::errors::QuickLendXError;
use crate::storage::InvoiceStorage;
use crate::types::{Dispute, DisputeStatus};
use crate::verification::{
    validate_dispute_eligibility, validate_dispute_evidence, validate_dispute_reason,
    validate_dispute_resolution,
};
use soroban_sdk::{symbol_short, Address, BytesN, Env, String, Vec};

fn dispute_index_key() -> soroban_sdk::Symbol {
    symbol_short!("dispute")
}

fn get_dispute_index(env: &Env) -> Vec<BytesN<32>> {
    env.storage()
        .instance()
        .get(&dispute_index_key())
        .unwrap_or_else(|| Vec::new(env))
}

fn add_to_dispute_index(env: &Env, invoice_id: &BytesN<32>) {
    let mut ids = get_dispute_index(env);
    if !ids.iter().any(|id| id == *invoice_id) {
        ids.push_back(invoice_id.clone());
        env.storage().instance().set(&dispute_index_key(), &ids);
    }
}

/// Track an invoice ID in the dispute index.
pub(crate) fn track_dispute_invoice(env: &Env, invoice_id: &BytesN<32>) {
    add_to_dispute_index(env, invoice_id);
}

pub fn create_dispute(
    env: &Env,
    invoice_id: &BytesN<32>,
    creator: &Address,
    reason: &String,
    evidence: &String,
) -> Result<(), QuickLendXError> {
    creator.require_auth();

    let mut invoice =
        InvoiceStorage::get_invoice(env, invoice_id).ok_or(QuickLendXError::InvoiceNotFound)?;

    validate_dispute_reason(reason)?;
    validate_dispute_evidence(evidence)?;
    validate_dispute_eligibility(&invoice, creator)?;

    invoice.dispute_status = DisputeStatus::Disputed;
    invoice.dispute = Dispute {
        created_by: creator.clone(),
        created_at: env.ledger().timestamp(),
        reason: reason.clone(),
        evidence: evidence.clone(),
        resolution: String::from_str(env, ""),
        resolved_by: creator.clone(),
        resolved_at: 0,
    };

    InvoiceStorage::update_invoice(env, &invoice);
    add_to_dispute_index(env, invoice_id);
    Ok(())
}

pub fn put_dispute_under_review(
    env: &Env,
    admin: &Address,
    invoice_id: &BytesN<32>,
) -> Result<(), QuickLendXError> {
    AdminStorage::require_admin(env, admin)?;
    let mut invoice =
        InvoiceStorage::get_invoice(env, invoice_id).ok_or(QuickLendXError::InvoiceNotFound)?;

    if invoice.dispute_status != DisputeStatus::Disputed {
        return Err(QuickLendXError::DisputeNotFound);
    }

    invoice.dispute_status = DisputeStatus::UnderReview;
    InvoiceStorage::update_invoice(env, &invoice);
    Ok(())
}

pub fn resolve_dispute(
    env: &Env,
    admin: &Address,
    invoice_id: &BytesN<32>,
    resolution: &String,
) -> Result<(), QuickLendXError> {
    AdminStorage::require_admin(env, admin)?;
    validate_dispute_resolution(resolution)?;

    let mut invoice =
        InvoiceStorage::get_invoice(env, invoice_id).ok_or(QuickLendXError::InvoiceNotFound)?;

    if invoice.dispute_status != DisputeStatus::UnderReview {
        return Err(QuickLendXError::DisputeNotUnderReview);
    }

    invoice.dispute_status = DisputeStatus::Resolved;
    invoice.dispute.resolution = resolution.clone();
    invoice.dispute.resolved_by = admin.clone();
    invoice.dispute.resolved_at = env.ledger().timestamp();
    InvoiceStorage::update_invoice(env, &invoice);
    Ok(())
}

pub fn get_dispute_details(env: &Env, invoice_id: &BytesN<32>) -> Option<Dispute> {
    let invoice = InvoiceStorage::get_invoice(env, invoice_id)?;
    if invoice.dispute_status == DisputeStatus::None {
        None
    } else {
        Some(invoice.dispute)
    }
}

pub fn get_invoices_with_disputes(env: &Env) -> Vec<BytesN<32>> {
    get_dispute_index(env)
}

pub(crate) fn indexed_dispute_invoices(env: &Env) -> Vec<BytesN<32>> {
    get_dispute_index(env)
}

pub fn get_invoices_by_dispute_status(env: &Env, status: &DisputeStatus) -> Vec<BytesN<32>> {
    let mut result = Vec::new(env);
    for invoice_id in get_dispute_index(env).iter() {
        if let Some(invoice) = InvoiceStorage::get_invoice(env, &invoice_id) {
            if invoice.dispute_status == *status {
                result.push_back(invoice_id);
            }
        }
    }
    result
}

pub(crate) fn indexed_invoices_by_status(env: &Env, status: &DisputeStatus) -> Vec<BytesN<32>> {
    get_invoices_by_dispute_status(env, status)
}
