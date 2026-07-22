#![no_std]

//! AniPay Direct — Harvest Payment Escrow
//!
//! Locks a trader's USDC payment on-chain when a rice harvest is picked up,
//! and releases it to the farmer automatically the moment the cooperative's
//! warehouse validator confirms delivery. This removes the 30-45 day payment
//! delay that pushes smallholder farmers toward informal, high-interest loans.

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

#[cfg(test)]
mod test;

/// Storage keys used by the contract.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// One escrow record per harvest lot, keyed by a unique escrow id.
    Escrow(u64),
    /// The cooperative's warehouse validator address, set once at setup.
    Validator,
}

/// Represents a single harvest payment held in escrow.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Escrow {
    pub buyer: Address,   // the rice trader
    pub farmer: Address,  // the farmer being paid
    pub token: Address,   // the USDC (or other) token contract address
    pub amount: i128,     // amount locked in escrow
    pub delivered: bool,  // has the cooperative confirmed the harvest arrived
    pub released: bool,   // have funds been paid out to the farmer
}

#[contract]
pub struct HarvestEscrowContract;

#[contractimpl]
impl HarvestEscrowContract {
    /// Sets the cooperative's warehouse validator. Called once by the
    /// cooperative admin when the contract is deployed for a given season.
    /// Only this address will be able to confirm deliveries.
    pub fn set_validator(env: Env, validator: Address) {
        validator.require_auth();
        env.storage().instance().set(&DataKey::Validator, &validator);
    }

    /// Trader (buyer) creates an escrow for a harvest lot and deposits the
    /// agreed USDC amount into the contract's custody. Funds leave the
    /// trader's wallet immediately, guaranteeing the farmer will be paid.
    pub fn create_escrow(
        env: Env,
        escrow_id: u64,
        buyer: Address,
        farmer: Address,
        token: Address,
        amount: i128,
    ) {
        buyer.require_auth();
        assert!(amount > 0, "amount must be positive");
        assert!(
            !env.storage().instance().has(&DataKey::Escrow(escrow_id)),
            "escrow already exists for this id"
        );

        // Move funds from the trader into the contract's own balance.
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&buyer, &env.current_contract_address(), &amount);

        let escrow = Escrow {
            buyer,
            farmer,
            token,
            amount,
            delivered: false,
            released: false,
        };
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);
    }

    /// Cooperative validator confirms the harvest was physically received at
    /// the warehouse. This single call both marks delivery and releases the
    /// locked USDC straight to the farmer's wallet — no separate step needed.
    pub fn confirm_delivery(env: Env, escrow_id: u64) {
        let validator: Address = env
            .storage()
            .instance()
            .get(&DataKey::Validator)
            .expect("validator not set");
        validator.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        assert!(!escrow.released, "escrow already released");

        escrow.delivered = true;
        escrow.released = true;

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.farmer,
            &escrow.amount,
        );

        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);
    }

    /// Read-only lookup so the app can show buyer/farmer the current status
    /// of a harvest payment (pending, delivered, released).
    pub fn get_escrow(env: Env, escrow_id: u64) -> Escrow {
        env.storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found")
    }
}
