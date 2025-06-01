#![no_std]
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Symbol, Vec};

#[contracttype]
pub struct EscrowedBalance {
    account: Address,
    amount: i128,
    claim_after: u64,
}

#[contracttype]
pub struct EscrowLockEvent {
    amount: i128,
    claim_after: u64,
}

#[contracttype]
pub struct EscrowUnlockEvent {
    amount: i128,
}

#[contracttype]
pub struct EscrowDetails {
    account: Address,
    amount: i128,
    claim_after: u64,
    can_unlock: bool,
}

#[contracttype]
pub enum DataKey {
    Token,
    MaxLockupDuration,
    Escrows,
    Escrow(Address),
}

#[contracterror]
pub enum Errors {
    ClaimAfterInPast = 1,
    LockupTooLong = 2,
    TooEarlyToUnlock = 3,
    EscrowNotFound = 4,
    EscrowAlreadyExists = 5,
}

#[contract]
pub struct Escrow;

#[contractimpl]
impl Escrow {
    pub fn __constructor(env: Env, token: Address, max_lockup_duration: u64) {
        if env.storage().instance().has(&DataKey::Token) {
            panic!("Contract already initialized");
        }
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::MaxLockupDuration, &max_lockup_duration);
        let escrows: Vec<Address> = Vec::new(&env);
        env.storage().persistent().set(&DataKey::Escrows, &escrows);
    }

    pub fn lock(env: Env, account: Address, amount: i128, claim_after: u64) -> Result<(), Errors> {
        account.require_auth();
        let current_time = env.ledger().timestamp();
        if claim_after <= current_time {
            return Err(Errors::ClaimAfterInPast);
        }
        let max_lockup: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MaxLockupDuration)
            .unwrap();
        if claim_after - current_time > max_lockup {
            return Err(Errors::LockupTooLong);
        }
        let key = DataKey::Escrow(account.clone());
        if env.storage().persistent().has(&key) {
            return Err(Errors::EscrowAlreadyExists);
        }
        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token = TokenClient::new(&env, &token_addr);
        token.transfer(&account, &env.current_contract_address(), &amount);
        let escrow = EscrowedBalance {
            account: account.clone(),
            amount,
            claim_after,
        };
        env.storage().persistent().set(&key, &escrow);
        let mut escrows: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Escrows)
            .unwrap_or(Vec::new(&env));
        escrows.push_back(account.clone());
        env.storage().persistent().set(&DataKey::Escrows, &escrows);
        env.events().publish(
            (
                Symbol::new(&env, "escrow"),
                Symbol::new(&env, "lock"),
                account,
            ),
            EscrowLockEvent {
                amount,
                claim_after,
            },
        );
        Ok(())
    }

    pub fn unlock(env: Env, account: Address) -> Result<(), Errors> {
        account.require_auth();
        let key = DataKey::Escrow(account.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Errors::EscrowNotFound);
        }
        let escrow: EscrowedBalance = env.storage().persistent().get(&key).unwrap();
        let current_time = env.ledger().timestamp();
        if current_time < escrow.claim_after {
            return Err(Errors::TooEarlyToUnlock);
        }
        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token = TokenClient::new(&env, &token_addr);
        token.transfer(&env.current_contract_address(), &account, &escrow.amount);
        env.storage().persistent().remove(&key);
        let mut escrows: Vec<Address> = env.storage().persistent().get(&DataKey::Escrows).unwrap();
        if let Some(index) = escrows.iter().position(|a| a == account) {
            escrows.remove(index as u32);
            env.storage().persistent().set(&DataKey::Escrows, &escrows);
        } else {
            panic!("Account not found in Escrows list");
        }
        env.events().publish(
            (
                Symbol::new(&env, "escrow"),
                Symbol::new(&env, "unlock"),
                account,
            ),
            EscrowUnlockEvent {
                amount: escrow.amount,
            },
        );
        Ok(())
    }

    pub fn extend_ttl(env: Env, ttl: u32) -> Result<(), Errors> {
        env.storage().instance().extend_ttl(ttl, ttl);
        env.events().publish(
            (Symbol::new(&env, "escrow"), Symbol::new(&env, "extend_ttl")),
            ttl,
        );
        Ok(())
    }

    pub fn get_escrow(env: Env, account: Address) -> Option<EscrowDetails> {
        let key = DataKey::Escrow(account);
        if let Some(escrow) = env
            .storage()
            .persistent()
            .get::<DataKey, EscrowedBalance>(&key)
        {
            let current_time = env.ledger().timestamp();
            let can_unlock = current_time >= escrow.claim_after;
            Some(EscrowDetails {
                account: escrow.account,
                amount: escrow.amount,
                claim_after: escrow.claim_after,
                can_unlock,
            })
        } else {
            None
        }
    }

    pub fn get_escrows(env: Env) -> Vec<EscrowDetails> {
        let escrows: Vec<Address> = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::Escrows)
            .unwrap_or(Vec::new(&env));
        let current_time = env.ledger().timestamp();
        let mut details = Vec::new(&env);
        for account in escrows.iter() {
            let key = DataKey::Escrow(account.clone());
            if let Some(escrow) = env
                .storage()
                .persistent()
                .get::<DataKey, EscrowedBalance>(&key)
            {
                let can_unlock = current_time >= escrow.claim_after;
                let detail = EscrowDetails {
                    account: escrow.account,
                    amount: escrow.amount,
                    claim_after: escrow.claim_after,
                    can_unlock,
                };
                details.push_back(detail);
            }
        }
        details
    }
}
