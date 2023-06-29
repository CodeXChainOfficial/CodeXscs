#![no_std]
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::ptr_arg)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use core::ops::Deref;
// use crate::idna::{ToAscii, ToUnicode};

pub mod callback_module;
pub mod constant_module;
pub mod data_module;
pub mod nft_module;
pub mod price_oracle_module;
pub mod storage_module;
pub mod user_builtin;
pub mod utils_module;

use callback_module::*;

use constant_module::{
    DAY_IN_SECONDS, HOUR_IN_SECONDS, MIGRATION_PERIOD, MIN_IN_SECONDS, MONTH_IN_SECONDS,
    NFT_AMOUNT, SUB_DOMAIN_COST_IN_CENT, YEAR_IN_SECONDS,
};
use data_module::{DomainName, DomainNameAttributes, Reservation, SubDomain, Profile, SocialMedia, TextRecord, Wallets};

/// A contract that registers and manages domain names issuance on MultiversX
#[multiversx_sc::contract]
pub trait XnMain:
    nft_module::NftModule
    + callback_module::CallbackModule
    + storage_module::StorageModule
    + utils_module::UtilsModule
    + price_oracle_module::PriceOracleModule
{
    #[init]
    fn init(&self, oracle_address: ManagedAddress) {
        // Set the oracle contract address
        self.oracle_address().set(&oracle_address);

        let default_prices_in_usd_cents: [u64; 5] = [10000u64, 10000u64, 10000u64, 1000u64, 100];

        for (_i, price) in default_prices_in_usd_cents.iter().enumerate() {
            self.domain_length_to_yearly_rent_usd().push(&price);
        }

        // set default EGLD/USD price
        self._set_egld_price();
        // self.egld_usd_price().set(268000000000000 as u64);
        // set default Migration start time.
        self.migration_start_time().set(self.get_current_time());
        // Initialize the allowed top-level domain names
        let tld_mvx = ManagedBuffer::from("mvx");
        self.allowed_top_level_domains().push(&tld_mvx);
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint]
    fn issue_token(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer) {
        require!(self.nft_token_id().is_empty(), "Token already issued");

        let payment_amount = self.call_value().egld_value();
        self.send()
            .esdt_system_sc_proxy()
            .issue_non_fungible(
                payment_amount,
                &token_name,
                &token_ticker,
                NonFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_transfer_create_role: true,
                    can_change_owner: false,
                    can_upgrade: false,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback())
            .call_and_exit();
    }

    #[only_owner]
    #[endpoint(set_local_roles)]
    fn set_local_roles(&self) {
        require!(!self.nft_token_id().is_empty(), "Token not issued");

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                (&[EsdtLocalRole::NftCreate][..]).into_iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[payable("EGLD")]
    #[endpoint]
    fn register_or_renew(
        &self,
        domain_name: ManagedBuffer,
        period: u8,
        unit: u8, // 0: year, 1: month, 2: day, 3: hour, 4: min
        assign_to: OptionalValue<ManagedAddress>,
    ) {
        let (token, _, payment) = self.call_value().egld_or_single_esdt().into_tuple();
        let caller = self.blockchain().get_caller();
        require!(period > 0, "Duration (years) must be a positive integer");

        let unit_seconds = match unit {
            0 => YEAR_IN_SECONDS,
            1 => MONTH_IN_SECONDS,
            2 => DAY_IN_SECONDS,
            3 => HOUR_IN_SECONDS,
            4 => MIN_IN_SECONDS,
            _ => panic!("Wrong date unit"),
        };

        let period_secs: u64 = u64::from(period) * unit_seconds;

        let is_name_valid = self.is_name_valid(&domain_name);
        let is_name_valid_message = if is_name_valid.err().is_some() {
            is_name_valid.err().unwrap()
        } else {
            ""
        };
        require!(is_name_valid.is_ok(), is_name_valid_message);

        // no subdomains
        let parts = self.split_domain_name(&domain_name);
        require!(parts.len() == 2, "You can only register domain names");

        require!(
            self.can_claim(&caller, &domain_name),
            "name is not available for caller"
        );

        let price = self.rent_price(&domain_name, &period_secs);
        require!(price <= payment, "Insufficient EGLD Funds");

        let since = self.get_current_time();

        let domain_record_exists = !self.domain_name(&domain_name).is_empty();

        // NFT functionality
        if domain_record_exists {
            // require!(self.domain_name(&domain_name).get().expires_at + GRACE_PERIOD < since, "Domain already exists.");
            // require!(self.is_owner(&caller, &domain_name), "Permission denied.");
            let mut domain_record = self.domain_name(&domain_name).get();
            domain_record.expires_at = since + period_secs;
            self.domain_name(&domain_name).set(domain_record.clone());
        } else {
            // // Mint NFT for the new owner
            let attributes = DomainNameAttributes {
                expires_at: since + period_secs,
            };
            let nft_nonce;
            match assign_to {
                OptionalValue::Some(to) => {
                    nft_nonce = self.mint_nft(&to, &domain_name, &price, &attributes);
                }
                OptionalValue::None => {
                    nft_nonce = self.mint_nft(&caller, &domain_name, &price, &attributes);
                }
            }
            let new_domain_record = DomainName {
                name: domain_name.clone(),
                expires_at: attributes.expires_at,
                nft_nonce,
                profile: Option::None,
                social_media: Option::None,
                text_record: Option::None,
                wallets: Option::None,
            };

            self.domain_name(&domain_name)
                .set(new_domain_record.clone());
        }

        // self._update_primary_address(&domain_name, assign_to);

        // return extra EGLD if customer sent more than required
        if price < payment {
            let excess = payment - price;
            self.send().direct(&caller, &token, 0, &excess);
        }
    }

    #[endpoint]
    fn update_domain_profile(&self, domain_name: ManagedBuffer, profile: Profile<Self::Api>, social_media: SocialMedia<Self::Api>, text_record: ManagedVec<TextRecord<Self::Api>>, wallets: Wallets<Self::Api>) {
        let domain_record_exists = !self.domain_name(&domain_name).is_empty();
        require!(domain_record_exists, "Domain not exist");

        let caller = self.blockchain().get_caller();

        require!(
            self.is_owner(&caller, &domain_name),
            "Not Allowed!"
        );

        let mut domain = self.domain_name(&domain_name).get();
        domain.profile = Some(profile);
        domain.social_media = Some(social_media);
        domain.text_record = Some(text_record);
        domain.wallets = Some(wallets);
        self.domain_name(&domain_name).set(&domain);
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

    #[payable("EGLD")]
    #[endpoint]
    fn register_sub_domain(&self, sub_domain: ManagedBuffer, address: ManagedAddress) {
        let (token, _, payment) = self.call_value().egld_or_single_esdt().into_tuple();
        let caller = self.blockchain().get_caller();
        require!(self.is_owner(&caller, &sub_domain), "Not Allowed!");
        let primary_domain = self.get_primary_domain(&sub_domain).unwrap();
        let len = self.sub_domains(&primary_domain).len();
        let mut is_exist = false;
        for i in 1..len + 1 {
            let item = self.sub_domains(&primary_domain).get(i);
            if sub_domain == item.name {
                is_exist = true;
                break;
            }
        }

        require!(!is_exist, "Already registered");
        self._set_egld_price();
        let egld_usd_price = self.egld_usd_price().get();
        let price_egld = BigUint::from(SUB_DOMAIN_COST_IN_CENT)
            * BigUint::from(10_000_000_000_000_000u64)
            / BigUint::from(egld_usd_price);
        require!(price_egld <= payment, "Insufficient EGLD Funds");
        let new_sub_domain = SubDomain {
            name: sub_domain,
            address: address,
        };
        let _ = &mut self.sub_domains(&primary_domain).push(&new_sub_domain);
        // return extra EGLD if customer sent more than required
        if price_egld < payment {
            let excess = payment - price_egld;
            self.send().direct(&caller, &token, 0, &excess);
        }
    }

    #[endpoint]
    fn migrate_domain(&self, domain_name: ManagedBuffer) {
        let caller = self.blockchain().get_caller();
        let is_exist = !self.reservations(&domain_name).is_empty();
        require!(is_exist, "Domain not exist");
        let reservation = self.reservations(&domain_name).get();
        require!(reservation.reserved_for == caller, "Not owner");
        let migration_start_time = self.migration_start_time().get();
        let current_time = self.get_current_time();
        require!(
            current_time < migration_start_time + MIGRATION_PERIOD,
            "Period exceeded for migration"
        );
        // no subdomains
        let parts = self.split_domain_name(&domain_name);
        require!(parts.len() == 2, "You can only register domain names");
        let mut new_domain_name: ManagedBuffer = parts.get(0).deref().clone();
        new_domain_name.append(&ManagedBuffer::from(".mvx"));
        let domain_record_exists = !self.domain_name(&new_domain_name).is_empty();
        require!(!domain_record_exists, "Domain already migrated.");

        // // Mint NFT for the new owner
        let attributes = DomainNameAttributes {
            expires_at: reservation.until,
        };
        let nft_nonce = self.mint_nft(
            &caller,
            &new_domain_name,
            &BigUint::from(0 as u64),
            &attributes,
        );
        let new_domain_record = DomainName {
            name: new_domain_name.clone(),
            expires_at: attributes.expires_at,
            nft_nonce,
            profile: Option::None,
            social_media: Option::None,
            text_record: Option::None,
            wallets: Option::None,
        };

        self.domain_name(&new_domain_name)
            .set(new_domain_record.clone());
        self.reservations(&new_domain_name).clear();
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
    #[payable("*")]
    #[endpoint]
    fn transfer_domain(&self, domain_name: ManagedBuffer, new_owner: ManagedAddress) {
        let (token_id, token_nonce, amount) = self.call_value().egld_or_single_esdt().into_tuple();
        let nft_token_id_mapper = self.nft_token_id();
        let domain_name_mapper = self.domain_name(&domain_name);
        let caller = self.blockchain().get_caller();

        require!(&caller != &new_owner, "can't transfer domain");

        require!(!domain_name_mapper.is_empty(), "wrong domain name");

        require!(
            domain_name_mapper.get().nft_nonce == token_nonce,
            "Not Allowed!"
        );

        require!(!nft_token_id_mapper.is_empty(), "Token not issued!");

        require!(
            token_id == self.nft_token_id().get()
                && token_nonce == domain_name_mapper.get().nft_nonce,
            "wrong Nft"
        );

        self.send().direct_esdt(
            &new_owner,
            &token_id.unwrap_esdt(),
            token_nonce,
            &BigUint::from(NFT_AMOUNT),
        );
    }

    #[endpoint]
    fn remove_sub_domain(&self, sub_domain_name: ManagedBuffer) {
        let caller = self.blockchain().get_caller();

        require!(self.is_owner(&caller, &sub_domain_name), "Not Allowed!");

        let primary_domain = self.get_primary_domain(&sub_domain_name).unwrap();
        let mut sub_domain_mapper = self.sub_domains(&primary_domain);
        let len = sub_domain_mapper.len();

        let mut index = 0;
        // require!(
        //     primary_domain == ManagedBuffer::new_from_bytes(b"owner.mvx"),
        //     "primary domain wrong"
        // );
        // require!(
        //     sub_domain_mapper.get(1).name == ManagedBuffer::new_from_bytes(b"subdomain.owner.mvx"),
        //     "storage value error"
        // );
        // require!(
        //     sub_domain_name == ManagedBuffer::new_from_bytes(b"subdomain.owner.mvx"),
        //     "parameter value error"
        // );

        for i in 1..len + 1 {
            if sub_domain_mapper.get(i).name == sub_domain_name {
                index = i;
                break;
            }
        }

        require!(index > 0, "there is no sub domain to remove");
        sub_domain_mapper.swap_remove(index);
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
        self._fetch_egld_usd_prices();
    }
}
