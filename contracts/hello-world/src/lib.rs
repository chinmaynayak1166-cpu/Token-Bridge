#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, Symbol, symbol_short};

// Structure to track computing resource listings
#[contracttype]
#[derive(Clone)]
pub struct ComputeResource {
    pub provider: Address,      // Address of the computing power provider
    pub token_rate: u64,        // Tokens per hour of computing time
    pub available_hours: u64,   // Total hours of computing power available
    pub is_active: bool,        // Whether the resource is currently available
}

// Structure to track rental agreements
#[contracttype]
#[derive(Clone)]
pub struct RentalAgreement {
    pub renter: Address,        // Address of the person renting computing power
    pub provider: Address,      // Address of the provider
    pub hours_rented: u64,      // Number of hours rented
    pub total_tokens: u64,      // Total tokens paid
    pub start_time: u64,        // Timestamp when rental started
    pub is_completed: bool,     // Whether the rental is completed
}

// Mapping for resources
#[contracttype]
pub enum ResourceBook {
    Resource(Address)
}

// Mapping for rentals
#[contracttype]
pub enum RentalBook {
    Rental(u64)
}

// Counter for rental IDs
const RENTAL_COUNTER: Symbol = symbol_short!("R_COUNT");

#[contract]
pub struct TokenBridgeContract;

#[contractimpl]
impl TokenBridgeContract {
    
    // Function to list computing resources for rent
    pub fn list_resource(
        env: Env, 
        provider: Address, 
        token_rate: u64, 
        available_hours: u64
    ) {
        provider.require_auth();
        
        let resource = ComputeResource {
            provider: provider.clone(),
            token_rate,
            available_hours,
            is_active: true,
        };
        
        env.storage().instance().set(
            &ResourceBook::Resource(provider.clone()), 
            &resource
        );
        
        env.storage().instance().extend_ttl(5000, 5000);
        log!(&env, "Computing resource listed by provider: {:?}", provider);
    }
    
    // Function to rent computing power
    pub fn rent_resource(
        env: Env, 
        renter: Address, 
        provider: Address, 
        hours: u64
    ) -> u64 {
        renter.require_auth();
        
        let mut resource = Self::view_resource(env.clone(), provider.clone());
        
        if !resource.is_active || resource.available_hours < hours {
            log!(&env, "Resource not available or insufficient hours");
            panic!("Resource not available");
        }
        
        let total_tokens = hours * resource.token_rate;
        let rental_id = env.storage().instance().get(&RENTAL_COUNTER).unwrap_or(0u64) + 1;
        let start_time = env.ledger().timestamp();
        
        let rental = RentalAgreement {
            renter: renter.clone(),
            provider: provider.clone(),
            hours_rented: hours,
            total_tokens,
            start_time,
            is_completed: false,
        };
        
        // Update resource availability
        resource.available_hours -= hours;
        if resource.available_hours == 0 {
            resource.is_active = false;
        }
        
        env.storage().instance().set(&ResourceBook::Resource(provider.clone()), &resource);
        env.storage().instance().set(&RentalBook::Rental(rental_id), &rental);
        env.storage().instance().set(&RENTAL_COUNTER, &rental_id);
        
        env.storage().instance().extend_ttl(5000, 5000);
        log!(&env, "Rental agreement created with ID: {}", rental_id);
        
        rental_id
    }
    
    // Function to complete a rental and mark it as finished
    pub fn complete_rental(env: Env, rental_id: u64, provider: Address) {
        provider.require_auth();
        
        let mut rental = Self::view_rental(env.clone(), rental_id);
        
        if rental.provider != provider {
            log!(&env, "Only the provider can complete this rental");
            panic!("Unauthorized");
        }
        
        if rental.is_completed {
            log!(&env, "Rental already completed");
            panic!("Already completed");
        }
        
        rental.is_completed = true;
        env.storage().instance().set(&RentalBook::Rental(rental_id), &rental);
        
        env.storage().instance().extend_ttl(5000, 5000);
        log!(&env, "Rental ID {} marked as completed", rental_id);
    }
    
    // Function to view resource details
    pub fn view_resource(env: Env, provider: Address) -> ComputeResource {
        env.storage().instance().get(&ResourceBook::Resource(provider.clone()))
            .unwrap_or(ComputeResource {
                provider: provider.clone(),
                token_rate: 0,
                available_hours: 0,
                is_active: false,
            })
    }
    
    // Function to view rental agreement details
    pub fn view_rental(env: Env, rental_id: u64) -> RentalAgreement {
        env.storage().instance().get(&RentalBook::Rental(rental_id))
            .unwrap_or(RentalAgreement {
                renter: Address::from_string(&env.string_from_str("")),
                provider: Address::from_string(&env.string_from_str("")),
                hours_rented: 0,
                total_tokens: 0,
                start_time: 0,
                is_completed: false,
            })
    }
}