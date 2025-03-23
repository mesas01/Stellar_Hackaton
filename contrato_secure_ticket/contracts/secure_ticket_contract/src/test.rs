#![cfg(test)]
extern crate std;

use crate::{SecureTicketContract, SecureTicketContractClient}; 

use soroban_sdk::{
    testutils::Address as _,
    token::Client as TokenClient,
    token::StellarAssetClient,
    Address, Env,
};


#[test]
fn test_initialize_and_create_ticket() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    let amount = 1_000_000_000;
    stellar_token.mint(&admin, &amount);
    token_client.transfer(&admin, &organizer, &amount);

    client.initialize(&organizer, &token_contract.address());

    let event_id = 1001;
    let price = 1_000_000;
    client.create_ticket(&event_id, &price);

    let ticket = client.get_ticket(&0);
    assert_eq!(ticket.id, 0);
    assert_eq!(ticket.event_id, event_id);
    assert_eq!(ticket.price, price);
    assert_eq!(ticket.owner, organizer);
    assert!(!ticket.for_sale);
    assert!(!ticket.is_resale);
}

#[test]
fn test_get_event_tickets() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    let amount = 1_000_000_000;
    stellar_token.mint(&admin, &amount);
    token_client.transfer(&admin, &organizer, &amount);

    client.initialize(&organizer, &token_contract.address());

    client.create_ticket(&42, &1_000_000);
    client.create_ticket(&42, &1_200_000);
    client.create_ticket(&100, &1_500_000);

    let event_tickets = client.get_event_tickets(&42);
    assert_eq!(event_tickets.len(), 2);
    assert_eq!(event_tickets.get(0).unwrap().event_id, 42);
    assert_eq!(event_tickets.get(1).unwrap().event_id, 42);
}

#[test]
fn test_get_resale_tickets() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    let amount = 1_000_000_000;
    stellar_token.mint(&admin, &amount);
    token_client.transfer(&admin, &organizer, &amount);

    client.initialize(&organizer, &token_contract.address());

    client.create_ticket(&1, &1_000_000);
    client.create_ticket(&1, &1_500_000);
    client.resell_ticket(&0, &2_000_000);

    let resale = client.get_resale_tickets();
    assert_eq!(resale.len(), 1);
    assert_eq!(resale.get(0).unwrap().id, 0);
    assert_eq!(resale.get(0).unwrap().price, 2_000_000);
}

#[test]
fn test_create_ticket_with_zero_price() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    let amount = 1_000_000_000;
    stellar_token.mint(&admin, &amount);
    token_client.transfer(&admin, &organizer, &amount);

    client.initialize(&organizer, &token_contract.address());

    client.create_ticket(&123, &0);
    let ticket = client.get_ticket(&0);
    assert_eq!(ticket.price, 0);
    assert_eq!(ticket.event_id, 123);
}

#[test]
#[should_panic(expected = "already_init")]
fn test_initialize_twice_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    let amount = 1_000_000_000;
    stellar_token.mint(&admin, &amount);
    token_client.transfer(&admin, &organizer, &amount);

    client.initialize(&organizer, &token_contract.address());
    client.initialize(&organizer, &token_contract.address()); // should panic
}

#[test]
#[should_panic(expected = "already_sale")]
fn test_resell_ticket_twice_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    let amount = 1_000_000_000;
    stellar_token.mint(&admin, &amount);
    token_client.transfer(&admin, &organizer, &amount);

    client.initialize(&organizer, &token_contract.address());

    client.create_ticket(&1, &1_000_000);
    client.resell_ticket(&0, &2_000_000);
    client.resell_ticket(&0, &2_000_000); // should panic
}

#[test]
#[should_panic(expected = "Ticket not found")]
fn test_get_nonexistent_ticket_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    let amount = 1_000_000_000;
    stellar_token.mint(&admin, &amount);
    token_client.transfer(&admin, &organizer, &amount);

    client.initialize(&organizer, &token_contract.address());

    // No se han creado tickets. Este debe hacer panic.
    client.get_ticket(&999);
}

#[test]
#[should_panic(expected = "invalid_price")]
fn test_create_ticket_with_negative_price_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    let amount = 1_000_000_000;
    stellar_token.mint(&admin, &amount);
    token_client.transfer(&admin, &organizer, &amount);

    client.initialize(&organizer, &token_contract.address());

    client.create_ticket(&999, &-1); // Esto debe causar un panic
}

#[test]
fn test_buy_ticket_not_for_sale_panics() {
    let result = std::panic::catch_unwind(|| {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(SecureTicketContract, ());
        let client = SecureTicketContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let organizer = Address::generate(&env);
        let buyer = Address::generate(&env);

        let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
        let token_client = TokenClient::new(&env, &token_contract.address());
        let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

        let amount = 1_000_000_000;
        stellar_token.mint(&admin, &amount);
        token_client.transfer(&admin, &organizer, &amount);
        token_client.transfer(&admin, &buyer, &amount); // Buyer tiene fondos

        client.initialize(&organizer, &token_contract.address());

        // Crear ticket pero sin ponerlo a la venta
        client.create_ticket(&1, &1_000_000);

        // Este debe causar un panic
        client.buy_ticket(&0, &buyer);
    });

    assert!(
        result.is_err(),
        "Expected a panic when buying a ticket not for sale, but none occurred"
    );
}

#[test]
fn test_buy_ticket_transfers_token() {
    let env = Env::default();
    env.mock_all_auths(); 

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    let initial_amount = 1_000_000_000;

    // Fondos
    stellar_token.mint(&seller, &initial_amount);
    stellar_token.mint(&buyer, &initial_amount);

    client.initialize(&organizer, &token_contract.address());

    // El organizador crea ticket
    client.create_ticket(&100, &1_000_000);

    // ⚠️ Se pone en venta para poder ser comprado
    client.resell_ticket(&0, &1_000_000);

    // El seller lo compra primero (simula venta primaria)
    client.buy_ticket(&0, &seller);

    // El seller lo revende
    client.resell_ticket(&0, &1_000_000);

    let organizer_balance_before = token_client.balance(&organizer);
    let seller_balance_before = token_client.balance(&seller);
    let buyer_balance_before = token_client.balance(&buyer);

    // El buyer compra en reventa
    client.buy_ticket(&0, &buyer);

    let updated_ticket = client.get_ticket(&0);
    assert_eq!(updated_ticket.owner, buyer);
    assert!(!updated_ticket.for_sale);
    assert!(!updated_ticket.is_resale);

    let organizer_fee = 1_000_000 * 30 / 100;
    let seller_amount = 1_000_000 - organizer_fee;

    let organizer_balance_after = token_client.balance(&organizer);
    let seller_balance_after = token_client.balance(&seller);
    let buyer_balance_after = token_client.balance(&buyer);

    assert_eq!(organizer_balance_after, organizer_balance_before + organizer_fee);
    assert_eq!(seller_balance_after, seller_balance_before + seller_amount);
    assert_eq!(buyer_balance_after, buyer_balance_before - 1_000_000);
}

#[test]
fn test_primary_sale_transfers_token() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let buyer = Address::generate(&env);

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    let initial_amount = 1_000_000_000;
    stellar_token.mint(&buyer, &initial_amount);

    client.initialize(&organizer, &token_contract.address());

    // 🎟️ El organizador crea y publica un ticket para venta primaria
    client.create_ticket(&200, &2_000_000);
    client.resell_ticket(&0, &2_000_000); // ✅ Ponerlo a la venta

    let organizer_balance_before = token_client.balance(&organizer);
    let buyer_balance_before = token_client.balance(&buyer);

    // 💸 El buyer compra el ticket
    client.buy_ticket(&0, &buyer);

    let ticket = client.get_ticket(&0);
    assert_eq!(ticket.owner, buyer);
    assert!(!ticket.for_sale);
    assert!(!ticket.is_resale); // ✅ No es reventa

    let organizer_balance_after = token_client.balance(&organizer);
    let buyer_balance_after = token_client.balance(&buyer);

    // ✅ En venta primaria, el 100% va al organizador
    assert_eq!(organizer_balance_after, organizer_balance_before + 2_000_000);
    assert_eq!(buyer_balance_after, buyer_balance_before - 2_000_000);
}

#[test]
#[should_panic] // Esto se debe al error que lanza el contrato de token
fn test_buy_ticket_with_insufficient_funds_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SecureTicketContract, ());
    let client = SecureTicketContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let seller = Address::generate(&env);
    let poor_buyer = Address::generate(&env); // comprador sin fondos

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let _token_client = TokenClient::new(&env, &token_contract.address());
    let stellar_token = StellarAssetClient::new(&env, &token_contract.address());

    // Seller y Admin reciben tokens, pero no el buyer
    let initial_amount = 1_000_000_000;
    stellar_token.mint(&seller, &initial_amount);

    client.initialize(&organizer, &token_contract.address());

    client.create_ticket(&77, &2_000_000);
    client.resell_ticket(&0, &2_000_000);

    // 🧑‍🦱 Buyer sin saldo intenta comprarlo
    client.buy_ticket(&0, &poor_buyer); // 💥 esto debe fallar
}
