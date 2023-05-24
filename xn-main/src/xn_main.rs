#![no_std]
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::ptr_arg)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// use crate::idna::{ToAscii, ToUnicode};

pub mod user_builtin;
pub mod nft_module;
pub mod callback_module;
pub mod storage_module;
pub mod utils_module;
pub mod data_module;
pub mod constant_module;

use callback_module::*;

use data_module::{Reservation, DomainName, DomainNameAttributes};
use constant_module::{YEAR_IN_SECONDS};

/// A contract that registers and manages domain names issuance on MultiversX
#[multiversx_sc::contract]
pub trait XnMain: 
    nft_module::NftModule 
    + callback_module::CallbackModule 
    + storage_module::StorageModule 
    + utils_module::UtilsModule {

    #[init]
    fn init(
        &self,
        oracle_address: ManagedAddress
    ) {
        // Set the oracle contract address
        self.oracle_address().set(&oracle_address);

        let default_prices_in_usd_cents: [u64; 5] = [10000u64, 10000u64, 10000u64, 1000u64, 100];
        
        for (_i, price) in default_prices_in_usd_cents.iter().enumerate() {
            self.domain_length_to_yearly_rent_usd().push(&price);
        }

        // set default EGLD/USD price
        self.egld_usd_price().set(268000000000000 as u64);

        // Initialize the allowed top-level domain names
        let tld_mvx = ManagedBuffer::from("mvx");
        self.allowed_top_level_domains().push(&tld_mvx);
    }

    // endpoints
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint]
    fn issue_token(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer) {
        require!(self.nft_token_id().is_empty(), "Token already issued");

        let payment_amount = self.call_value().egld_value();
        let props = NonFungibleTokenProperties {
            can_freeze: true,
            can_wipe: true,
            can_pause: true,
            can_transfer_create_role: true,
            can_change_owner: false,
            can_upgrade: true,
            can_add_special_roles: true,
        };
        self.send()
            .esdt_system_sc_proxy()
            .issue_non_fungible(payment_amount, &token_name, &token_ticker, props)
            .async_call()
            .with_callback(self.callbacks().issue_callback())
            .call_and_exit();
    }

    #[payable("EGLD")]
    #[endpoint]
    fn register_or_renew(
        &self,
        domain_name: ManagedBuffer,
        years: u8,
        assign_to: OptionalValue<ManagedAddress>,
    ) {
        let (token, _, payment) = self.call_value().egld_or_single_esdt().into_tuple();

        let caller = self.blockchain().get_caller();

        require!(years > 0, "Duration (years) must be a positive integer");

        let is_name_valid = self.is_name_valid(&domain_name);
        let is_name_valid_message = if is_name_valid.err().is_some() {
            is_name_valid.err().unwrap()
        } else {
            ""
        };
        require!(is_name_valid.is_ok(), is_name_valid_message);

        // no subdomains, no TLDs accepted.
        let parts = self.split_domain_name(&domain_name);
        require!(parts.len() == 2, "You can only register domain names");

        require!(
            self.can_claim(&caller, &domain_name),
            "name is not available for caller"
        );

        let price = self.rent_price(&domain_name, &years);
        require!(price <= payment, "Insufficient EGLD Funds");

        let mut since = self.get_current_time();

        let domain_record_exists = !self.domain_name(&domain_name).is_empty();

        if domain_record_exists {
            since = self.domain_name(&domain_name).get().expires_at;
        }

        // NFT functionality
        if domain_record_exists {
            let nft_nonce = self.domain_name(&domain_name).get().nft_nonce;
            // Burn NFT for the previous owner
            self.burn_nft(nft_nonce);
        }

        // // Mint NFT for the new owner
        let attributes = DomainNameAttributes {
            expires_at: since + (u64::from(years) * YEAR_IN_SECONDS),
        };

        let nft_nonce = self.mint_nft(&caller, &domain_name, &price, &attributes);

        let new_domain_record = DomainName {
            name: domain_name.clone(),
            expires_at: attributes.expires_at,
            nft_nonce,
        };

        self.domain_name(&domain_name)
            .set(new_domain_record.clone());
        self._update_primary_address(&domain_name, assign_to);
        self.owner_domain_name(&domain_name).set(caller.clone());

        // return extra EGLD if customer sent more than required
        if price < payment {
            let excess = payment - price;
            self.send().direct(&caller, &token, 0, &excess);
        }
    }

    #[endpoint]
    fn update_primary_address(
        &self,
        domain_name_or_sub_domain: ManagedBuffer,
        assign_to: OptionalValue<ManagedAddress>,
    ) {
        let caller = self.blockchain().get_caller();

        require!(
            self.is_owner(&caller, &domain_name_or_sub_domain),
            "Not Allowed!"
        );

        self._update_primary_address(&domain_name_or_sub_domain, assign_to);
    }

    #[endpoint]
    fn update_key_value(
        &self,
        domain_name: ManagedBuffer,
        key: ManagedBuffer,
        value: OptionalValue<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();

        require!(
            self.is_owner(&caller, &domain_name)
                || self.resolve_domain_name(&domain_name).get() == caller.clone(),
            "Not Allowed!"
        );

        match value {
            OptionalValue::Some(_value) => {
                self.resolve_domain_name_key(&domain_name, &key).set(_value)
            }
            OptionalValue::None => {
                self.resolve_domain_name_key(&domain_name, &key).clear();
            }
        }
    }

    #[endpoint]
    fn accept(&self, domain_name: ManagedBuffer) {
        let caller = self.blockchain().get_caller();

        // caller has to have a acceptRequest matching the domain_name
        require!(
            self.accept_request(&domain_name).get() == caller.clone(),
            "Caller doesn't have acceptRequest for requested domain name"
        );

        self._set_resolve_doamin(&domain_name, &caller);
        self.accept_request(&domain_name).clear();
    }

    #[endpoint]
    fn revoke_accept_request(&self, domain_name: ManagedBuffer) {
        let caller = self.blockchain().get_caller();

        require!(
            self.is_owner(&caller, &domain_name)
                || self.accept_request(&domain_name).get() == caller.clone(),
            "Not Allowed!"
        );

        self.accept_request(&domain_name).clear();
    }

    // endpoints - admin-only
    #[only_owner]
    #[endpoint]
    fn set_reservations(&self, reservations: ManagedVec<Reservation<Self::Api>>) {
        for reservation in reservations.iter() {
            self.reservations(&reservation.domain_name).set(reservation);
        }
    }

    #[only_owner]
    #[endpoint]
    fn clear_reservations(&self, domain_names: ManagedVec<ManagedBuffer<Self::Api>>) {
        for domain_name in domain_names.iter() {
            self.reservations(&domain_name).clear();
        }
    }

    #[only_owner]
    #[endpoint]
    fn update_price_usd(&self, domain_length: u64, yearly_rent_usd: u64) {
        // Update the storage with the new price
        self.domain_length_to_yearly_rent_usd()
            .set(domain_length as usize, &yearly_rent_usd);
    }

    #[only_owner]
    #[endpoint]
    fn fetch_egld_usd_prices(&self) {
        let oracle_address = self.oracle_address().get();
        let mut args = ManagedVec::new();
        args.push(ManagedBuffer::from("egld"));
        args.push(ManagedBuffer::from("usd"));

        self.send()
            .contract_call::<()>(oracle_address, ManagedBuffer::from("latestPriceFeed"))
            .with_raw_arguments(args.into())
            .async_call()
            .with_callback(self.callbacks().fetch_egld_usd_prices_callback())
            .call_and_exit()
    }
}
