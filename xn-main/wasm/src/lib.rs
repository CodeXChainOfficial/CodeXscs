// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           13
// Async Callback (empty):               1
// Total number of exported functions:  15

#![no_std]
#![feature(alloc_error_handler, lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    xn_main
    (
        register_or_renew
        update_primary_address
        update_key_value
        accept
        revokeAcceptRequest
        reserve
        set_prices
        get_accept_request
        get_domain_name
        get_owner_domain_name
        resolve
        resolve_domain_name_key
        get_prices_egld
    )
}

multiversx_sc_wasm_adapter::empty_callback! {}
