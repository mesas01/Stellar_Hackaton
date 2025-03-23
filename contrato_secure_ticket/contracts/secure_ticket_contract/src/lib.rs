// src/lib.rs
#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, Address, Vec};
use soroban_sdk::token::Client as TokenClient;

const COMMISSION_PERCENT: i128 = 30;
const PERCENT_BASE: i128 = 100;

#[derive(Clone)]
#[contracttype]
pub struct Ticket {
    pub id: u32,
    pub event_id: u32,
    pub owner: Address,
    pub price: i128,
    pub for_sale: bool,
    pub is_resale: bool,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Ticket(u32),
    TicketCount,
    Owner,
    Token,
}

#[contract]
pub struct SecureTicketContract;

#[contractimpl]
impl SecureTicketContract {
    pub fn initialize(env: Env, owner: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Owner) {
            panic!("already_init");
        }
        owner.require_auth();
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::TicketCount, &0u32);
        env.storage().instance().set(&DataKey::Token, &token);
    }

    pub fn create_ticket(env: Env, event_id: u32, price: i128) {
        if price < 0 {
            panic!("invalid_price");
        }
    
        // 🔒 Obtener el dueño original del contrato (organizador)
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("Contract not initialized");
        owner.require_auth();
    
        // 🧾 Obtener contador actual de tickets
        let mut ticket_count: u32 = env.storage().instance().get(&DataKey::TicketCount).unwrap_or(0);
        let ticket_id = ticket_count;
    
        // 🎟️ Crear el ticket
        let ticket = Ticket {
            id: ticket_id,
            event_id,
            owner: owner.clone(),
            price,
            for_sale: false,
            is_resale: false,
        };
    
        // 💾 Guardar ticket y actualizar contador
        env.storage().instance().set(&DataKey::Ticket(ticket_id), &ticket);
        ticket_count += 1;
        env.storage().instance().set(&DataKey::TicketCount, &ticket_count);
    }
         

    pub fn get_ticket(env: Env, ticket_id: u32) -> Ticket {
        env.storage()
            .instance()
            .get(&DataKey::Ticket(ticket_id))
            .expect("Ticket not found")
    }

    pub fn get_owner(env: Env, ticket_id: u32) -> Address {
        let ticket: Ticket = Self::get_ticket(env, ticket_id);
        ticket.owner
    }

    pub fn resell_ticket(env: Env, ticket_id: u32, new_price: i128) {
        let mut ticket: Ticket = env.storage()
            .instance()
            .get(&DataKey::Ticket(ticket_id))
            .expect("Ticket not found");

        ticket.owner.require_auth();

        if ticket.for_sale {
            panic!("already_sale");
        }

        ticket.price = new_price;
        ticket.for_sale = true;
        ticket.is_resale = true;

        env.storage().instance().set(&DataKey::Ticket(ticket_id), &ticket);
    }

    pub fn buy_ticket(env: Env, ticket_id: u32, buyer: Address) {
        let ticket_opt = env
            .storage()
            .instance()
            .get::<_, Ticket>(&DataKey::Ticket(ticket_id));
    
        if ticket_opt.is_none() {
            panic!("Ticket not found");
        }
    
        let mut ticket = ticket_opt.unwrap();
    
        if !ticket.for_sale {
            panic!("not_sale");
        }
    
        buyer.require_auth();
    
        let organizer: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("Contract not initialized");
    
        let token: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .expect("Token not set");
    
        let token_client = TokenClient::new(&env, &token);
    
        if ticket.is_resale {
            let organizer_fee = ticket.price * COMMISSION_PERCENT / PERCENT_BASE;
            let seller_amount = ticket.price - organizer_fee;
    
            token_client.transfer(&buyer, &organizer, &organizer_fee);
            token_client.transfer(&buyer, &ticket.owner, &seller_amount);
        } else {
            token_client.transfer(&buyer, &organizer, &ticket.price);
        }
    
        ticket.owner = buyer.clone();
        ticket.for_sale = false;
        ticket.is_resale = false;
    
        env.storage().instance().set(&DataKey::Ticket(ticket_id), &ticket);
    }               

    pub fn get_resale_tickets(env: Env) -> Vec<Ticket> {
        let ticket_count: u32 = env.storage().instance().get(&DataKey::TicketCount).unwrap_or(0);
        let mut resale_tickets = Vec::new(&env);

        for id in 0..ticket_count {
            if let Some(ticket) = env.storage().instance().get::<_, Ticket>(&DataKey::Ticket(id)) {
                if ticket.for_sale && ticket.is_resale {
                    resale_tickets.push_back(ticket);
                }
            }
        }

        resale_tickets
    }

    pub fn get_event_tickets(env: Env, event_id: u32) -> Vec<Ticket> {
        let ticket_count: u32 = env.storage().instance().get(&DataKey::TicketCount).unwrap_or(0);
        let mut event_tickets = Vec::new(&env);

        for id in 0..ticket_count {
            if let Some(ticket) = env.storage().instance().get::<_, Ticket>(&DataKey::Ticket(id)) {
                if ticket.event_id == event_id {
                    event_tickets.push_back(ticket);
                }
            }
        }

        event_tickets
    }
}

#[cfg(test)]
mod test;
