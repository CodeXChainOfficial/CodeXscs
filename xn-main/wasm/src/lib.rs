// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           28
// Async Callback:                       1
// Total number of exported functions:  30

#![no_std]
#![feature(alloc_error_handler, lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    xn_main
    (
        register_or_renew
        update_domain_profile_overview
        update_domain_profile_socialmedia
        update_domain_profile_textrecord
        update_domain_profile_wallets
        update_primary_address
        register_sub_domain
        migrate_domain
        update_key_value
        transfer_domain
        remove_sub_domain
        set_reservations
        clear_reservations
        update_price_usd
        fetch_egld_usd_prices
        get_reservation
        get_domain_nft
        get_accept_request
        get_domain_name
        get_sub_domains
        get_owner_domain_name
        resolve
        resolve_domain_name_key
        get_prices_usd
        get_egld_usd_price
        get_allowed_top_level_domains
        get_migration_start_time
        get_royalties
        callBack
    )
}
