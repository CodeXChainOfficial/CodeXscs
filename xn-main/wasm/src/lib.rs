// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           30
// Async Callback:                       1
// Total number of exported functions:  32

#![no_std]
#![feature(alloc_error_handler, lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    xn_main
    (
        issue_and_set_all_roles
        register_domain
        extend_domain
        update_domain_overview
        update_domain_socials
        update_domain_wallets
        update_domain_textrecord
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
        get_domain_nft_id
        get_reservation
        get_domain_nft
        get_domain
        get_sub_domains
        get_main_domain
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
