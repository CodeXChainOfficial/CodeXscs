#![no_std]
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::ptr_arg)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// use crate::idna::{ToAscii, ToUnicode};

use multiversx_sc::types::heap::String;
use multiversx_sc::types::heap::Vec;

pub mod user_builtin;

#[macro_use]
extern crate alloc;

const GRACE_PERIOD: u64 = 21 * 24 * 60 * 60; // 21 days
const YEAR_IN_SECONDS: u64 = 365 * 24 * 60 * 60; // 1 year (365 days)
const MIN_LENGTH: usize = 3;
const MAX_LENGTH: usize = 256;

const NFT_AMOUNT: u32 = 1;

// objects
#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct DomainName<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub expires_at: u64,
    pub nft_nonce: u64,
}

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct DomainNameAttributes {
    pub expires_at: u64,
}

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct AcceptRequest<M: ManagedTypeApi> {
    pub domain_name: DomainName<M>,
    pub until: u64,
}

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct Reservation<M: ManagedTypeApi> {
    pub domain_name: ManagedBuffer<M>,
    pub until: u64,
    pub reserved_for: ManagedAddress<M>,
}

#[allow(clippy::manual_range_contains)]
fn check_name_char(ch: u8) -> bool {
    if ch >= b'a' && ch <= b'z' {
        return true;
    }

    if ch >= b'0' && ch <= b'9' {
        return true;
    }

    if ch == b'.' {
        return true;
    }

    false
}

/// A contract that registers and manages domain names issuance on MultiversX
#[multiversx_sc::contract]
pub trait XnMain {
    #[init]
    fn init(
        &self,
        oracle_address: ManagedAddress,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
    ) {
        // Set the oracle contract address
        self.oracle_address().set(&oracle_address);

        let DEFAULT_PRICES_IN_USD_CENTS: [u64; 5] = [10000u64, 10000u64, 10000u64, 1000u64, 100];

        for (i, price) in DEFAULT_PRICES_IN_USD_CENTS.iter().enumerate() {
            self.domain_length_to_yearly_rent_usd().push(&price);
        }

        // set default EGLD/USD price
        self.egld_usd_price().set(268000000000000 as u64);

        // Initialize the allowed top-level domain names
        let tld_mvx = ManagedBuffer::from("mvx");
        self.allowed_top_level_domains().push(&tld_mvx);
    }

    // endpoints
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

    /// validate_name upon registration
    fn is_name_valid(&self, name: &ManagedBuffer) -> Result<(), &'static str> {
        let name_len = name.len();

        if name_len <= MIN_LENGTH {
            return Result::Err("name too short");
        }

        if name_len > MAX_LENGTH {
            return Result::Err("name too long");
        }

        let mut name_bytes = [0u8; MAX_LENGTH];
        let name_slice: &mut [u8] = &mut name_bytes[..name_len];
        if name.load_slice(0, name_slice).is_err() {
            return Result::Err("error loading name bytes");
        }

        for ch in name_slice.iter() {
            if !check_name_char(*ch) {
                return Result::Err("character not allowed");
            }
        }

        // Check if the domain name has a valid top-level domain
        if let Some(tld_start) = name_slice.iter().rposition(|&ch| ch == b'.') {
            let tld = &name_slice[tld_start + 1..];
            let mut is_tld_valid = false;

            for allowed_tld in self.allowed_top_level_domains().iter() {
                let mut name_bytes = [0u8; MAX_LENGTH];
                let tld_slice: &mut [u8] = &mut name_bytes[..allowed_tld.len()];

                if allowed_tld.load_slice(0, tld_slice).is_err() {
                    return Result::Err("error loading tld bytes");
                }

                if tld_slice == tld {
                    is_tld_valid = true;
                    break;
                }
            }

            if !is_tld_valid {
                return Result::Err("invalid top-level domain");
            }
        } else {
            return Result::Err("missing top-level domain");
        }

        let name_str = match String::from_utf8(Vec::from(name_slice)) {
            Result::Ok(s) => s,
            Result::Err(_) => return Result::Err("name is not valid UTF-8"),
        };

        // TODO: would be nice to have punycode checks so we allow emojis and other langs
        // Problem: we need to import too many things

        // let decoded_name = match ToUnicode::to_unicode(name_str) {
        //     Ok(s) => s,
        //     Err(_) => return Result::Err("name contains invalid punycode characters"),
        // };

        // let encoded_name = match ToAscii::to_ascii(&decoded_name) {
        //     Ok(s) => s,
        //     Err(_) => return Result::Err("name cannot be converted to punycode"),
        // };

        // let encoded_name_slice = encoded_name.as_bytes();

        Result::Ok(())
    }

    // Helper function to split domain name into parts
    fn split_domain_name(&self, name: &ManagedBuffer) -> ManagedVec<ManagedBuffer> {
        let mut parts = ManagedVec::new();
        let mut start = 0;
        let mut name_bytes = [0u8; MAX_LENGTH];
        let name_len = name.len();
        let name_slice = &mut name_bytes[..name_len];

        name.load_slice(0, name_slice)
            .expect("error loading name bytes");

        for (i, &byte) in name_slice.iter().enumerate() {
            if byte == b'.' {
                parts.push(ManagedBuffer::from(&name_slice[start..i]));
                start = i + 1;
            }
        }
        parts.push(ManagedBuffer::from(&name_slice[start..]));

        parts
    }

    fn rent_price(&self, domain_name: &ManagedBuffer, years: &u8) -> BigUint<Self::Api> {
        let len = domain_name.len();
        let prices_len = self.domain_length_to_yearly_rent_usd().len();
        let price_index = if len < prices_len {
            len
        } else {
            prices_len - 1
        };

        let yearly_price_usd = self.domain_length_to_yearly_rent_usd().get(price_index);
        let egld_usd_price = self.egld_usd_price().get();

        let price_egld = BigUint::from(yearly_price_usd * u64::from(years.clone()))
            * BigUint::from(egld_usd_price);

        price_egld
    }

    fn get_current_time(&self) -> u64 {
        self.blockchain().get_block_timestamp()
    }

    fn can_claim(&self, address: &ManagedAddress, domain_name: &ManagedBuffer) -> bool {
        if self.is_owner(address, domain_name) {
            return true;
        }

        // Check if the address has a valid reservation for the domain name
        if !self.reservations(&domain_name).is_empty() {
            let res = self.reservations(&domain_name).get();
            if res.reserved_for == address.clone() {
                if self.get_current_time() <= res.until {
                    return true;
                } else {
                    self.reservations(&domain_name).clear();
                    return false;
                }
            }
        }

        // if domain name has expired
        if self.domain_name(&domain_name).is_empty() {
            return true;
        } else if self.get_current_time()
            >= self.domain_name(&domain_name).get().expires_at + GRACE_PERIOD
        {
            return true;
        }

        return false;
    }

    fn is_owner(&self, address: &ManagedAddress, domain_name: &ManagedBuffer) -> bool {
        let parts = self.split_domain_name(&domain_name);
        require!(parts.len() >= 2, "domain name is not valid");

        let domain_size = parts.get(parts.len() - 2).len() + 1 + parts.get(parts.len() - 1).len();

        let name = domain_name.copy_slice(domain_name.len() - domain_size, domain_size);
        match name {
            Option::Some(name) => {
                if !self.owner_domain_name(&name).is_empty()
                    && self.owner_domain_name(&name).get() == address.clone()
                {
                    return true;
                }
            }
            Option::None => {}
        }

        return false;
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

    fn _update_primary_address(
        &self,
        domain_name: &ManagedBuffer,
        assign_to: OptionalValue<ManagedAddress>,
    ) {
        let caller = self.blockchain().get_caller();

        if !self.resolve_domain_name(&domain_name).is_empty() {
            let current_address = self.resolve_domain_name(&domain_name).get();
            self.user_builtin_proxy(current_address.clone())
                .del_user_name()
                .async_call()
                .with_callback(self.callbacks().del_user_name_callback(&domain_name))
                .call_and_exit();
        }

        match assign_to {
            OptionalValue::Some(address) => {
                if address == caller {
                    self._set_resolve_doamin(&domain_name, &address);
                } else {
                    self.accept_request(&domain_name).set(address);
                }
            }
            OptionalValue::None => {}
        }
    }

    fn _set_resolve_doamin(&self, domain_name: &ManagedBuffer, address: &ManagedAddress) {
        self.user_builtin_proxy(address.clone())
            .set_user_name(domain_name.clone())
            .async_call()
            .with_callback(
                self.callbacks()
                    .set_user_name_callback(&domain_name, &address),
            )
            .call_and_exit();
    }

    #[callback]
    fn set_user_name_callback(
        &self,
        domain_name: &ManagedBuffer,
        address: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.resolve_domain_name(&domain_name).set(address);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[callback]
    fn del_user_name_callback(
        &self,
        domain_name: &ManagedBuffer,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.resolve_domain_name(&domain_name).clear();
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
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

    #[callback]
    fn fetch_egld_usd_prices_callback(&self, #[call_result] result: ManagedAsyncCallResult<u64>) {
        match result {
            ManagedAsyncCallResult::Ok(price) => {
                self.egld_usd_price().set(price);
            }
            ManagedAsyncCallResult::Err(_) => {
                // this can only fail if the oracle contract address is invalid
                // nothing to revert in case of error
            }
        }
    }

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

    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        self.require_token_issued();

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                [EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn][..]
                    .iter()
                    .cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[allow(clippy::too_many_arguments)]
    fn create_nft_with_attributes<T: TopEncode>(
        &self,
        name: ManagedBuffer,
        royalties: BigUint,
        attributes: T,
        uri: ManagedBuffer,
        selling_price: BigUint,
        token_used_as_payment: EgldOrEsdtTokenIdentifier,
        token_used_as_payment_nonce: u64,
    ) -> u64 {
        self.require_token_issued();
        let nft_token_id = self.nft_token_id().get();

        let mut serialized_attributes = ManagedBuffer::new();
        if let core::result::Result::Err(err) = attributes.top_encode(&mut serialized_attributes) {
            sc_panic!("Attributes encode error: {}", err.message_bytes());
        }

        let attributes_sha256 = self.crypto().sha256(&serialized_attributes);
        let attributes_hash = attributes_sha256.as_managed_buffer();
        let uris = ManagedVec::from_single_item(uri);
        let nft_nonce = self.send().esdt_nft_create(
            &nft_token_id,
            &BigUint::from(NFT_AMOUNT),
            &name,
            &royalties,
            attributes_hash,
            &attributes,
            &uris,
        );

        nft_nonce
    }

    fn require_token_issued(&self) {
        require!(!self.nft_token_id().is_empty(), "Token not issued");
    }

    fn burn_nft(&self, nft_nonce: u64) {
        let nft_token_id = self.nft_token_id().get();
        self.send()
            .esdt_local_burn(&nft_token_id, nft_nonce, &BigUint::from(NFT_AMOUNT));
    }

    fn get_owner_nft(&self, nft_nonce: u64) {
        let nft_token_id = self.nft_token_id().get();
        self.send()
            .esdt_local_burn(&nft_token_id, nft_nonce, &BigUint::from(NFT_AMOUNT));
    }

    fn mint_nft(
        &self,
        new_owner: &ManagedAddress,
        domain_name: &ManagedBuffer,
        selling_price: &BigUint,
        attributes: &DomainNameAttributes,
    ) -> u64 {
        let nft_token_id = self.nft_token_id().get();
        let name = domain_name.clone();
        let royalties = BigUint::zero();
        let uri = ManagedBuffer::new();
        let token_used_as_payment = EgldOrEsdtTokenIdentifier::egld();
        let token_used_as_payment_nonce = 0;

        let nft_nonce = self.create_nft_with_attributes(
            name,
            royalties,
            attributes,
            uri,
            selling_price.clone(),
            token_used_as_payment,
            token_used_as_payment_nonce,
        );

        self.send().direct_esdt(
            new_owner,
            &nft_token_id,
            nft_nonce,
            &BigUint::from(NFT_AMOUNT),
        );

        nft_nonce
    }

    #[callback]
    fn issue_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<EgldOrEsdtTokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.nft_token_id().set(&token_id.unwrap_esdt());
            }
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let returned = self.call_value().egld_or_single_esdt();
                if returned.token_identifier.is_egld() && returned.amount > 0 {
                    self.send()
                        .direct(&caller, &returned.token_identifier, 0, &returned.amount);
                }
            }
        }
    }

    // storage
    #[view(get_reservation)]
    #[storage_mapper("reservations")]
    fn reservations(
        &self,
        domain_name: &ManagedBuffer,
    ) -> SingleValueMapper<Reservation<Self::Api>>;

    #[view(getNftTokenId)]
    #[storage_mapper("nftTokenId")]
    fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(get_accept_request)]
    #[storage_mapper("accept_request")]
    fn accept_request(&self, domain_name: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[view(get_domain_name)]
    #[storage_mapper("domain_name")]
    fn domain_name(&self, domain_name: &ManagedBuffer) -> SingleValueMapper<DomainName<Self::Api>>;

    #[view(get_owner_domain_name)]
    #[storage_mapper("owner_domain_name")]
    fn owner_domain_name(&self, domain_name: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[view(resolve)]
    #[storage_mapper("resolve_domain_name")]
    fn resolve_domain_name(&self, domain_name: &ManagedBuffer)
        -> SingleValueMapper<ManagedAddress>;

    #[view(resolve_domain_name_key)]
    #[storage_mapper("resolve_key")]
    fn resolve_domain_name_key(
        &self,
        domain_name: &ManagedBuffer,
        key: &ManagedBuffer,
    ) -> SingleValueMapper<ManagedBuffer>;

    #[view(get_prices_usd)]
    #[storage_mapper("prices_usd")]
    fn domain_length_to_yearly_rent_usd(&self) -> VecMapper<u64>;

    #[view(get_egld_usd_price)]
    #[storage_mapper("egld_usd_price")]
    fn egld_usd_price(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("oracle_address")]
    fn oracle_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(get_allowed_top_level_domains)]
    #[storage_mapper("allowed_top_level_domains")]
    fn allowed_top_level_domains(&self) -> VecMapper<ManagedBuffer>;

    #[proxy]
    fn user_builtin_proxy(&self, to: ManagedAddress) -> user_builtin::Proxy<Self::Api>;
}
