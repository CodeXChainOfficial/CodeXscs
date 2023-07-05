#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use core::ops::Deref;

pub mod async_call_module;
pub mod callback_module;
pub mod constant_module;
pub mod data_module;
pub mod nft_module;
pub mod price_oracle_module;
pub mod storage_module;
pub mod user_builtin;
pub mod utils_module;

use constant_module::{MIGRATION_PERIOD, NFT_AMOUNT, WEGLD_ID};
use data_module::{
    DomainName, DomainNameAttributes, PeriodType, Profile, RentalFee, Reservation, SocialMedia,
    SubDomain, TextRecord, Wallets,
};

#[multiversx_sc::contract]
pub trait XnMain:
    nft_module::NftModule
    + async_call_module::AsyncCallModule
    + callback_module::CallbackModule
    + storage_module::StorageModule
    + utils_module::UtilsModule
    + price_oracle_module::PriceOracleModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(&self, oracle_address: ManagedAddress) {
        // Set the oracle contract address
        self.oracle_address().set(&oracle_address);

        //set default annual rental for domain name length in US cents
        let default_rent_fees = RentalFee {
            one_letter: 10_000u64,
            two_letter: 10_000u64,
            three_letter: 1u64,
            four_letter: 1_000u64,
            other: 100,
        };
        self.rental_fee().set(default_rent_fees);

        // set default royalties
        self.royalties().set(0);

        // set default Migration start time.
        self.migration_start_time().set(self.get_current_time());

        // Initialize the allowed top-level domain names
        let tld_mvx = ManagedBuffer::from("mvx");
        self.allowed_top_level_domains().push(&tld_mvx);

        // set default EGLD/USD price
        // self.egld_usd_price().set(268000000000000 as u64);
        self.internal_set_egld_price();
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint]
    fn issue_and_set_all_roles(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let issue_cost = self.call_value().egld_value();
        self.domain_nft().issue_and_set_all_roles(
            EsdtTokenType::NonFungible,
            issue_cost,
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[payable("EGLD")]
    #[endpoint]
    fn register_or_renew(
        &self,
        domain_name: ManagedBuffer,
        period: u8,
        unit: PeriodType,
        assign_to: OptionalValue<ManagedAddress>,
    ) {
        let caller = self.blockchain().get_caller();

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

        self.get_egld_price_for_register(domain_name, period, unit, assign_to);
    }

    #[payable("*")]
    #[endpoint]
    fn update_domain_profile(
        &self,
        domain_name: ManagedBuffer,
        profile: OptionalValue<Profile<Self::Api>>,
        social_media: OptionalValue<SocialMedia<Self::Api>>,
        text_record: OptionalValue<ManagedVec<TextRecord<Self::Api>>>,
        wallets: OptionalValue<Wallets<Self::Api>>,
    ) {
        let (token_id, token_nonce, _amount) = self.call_value().egld_or_single_esdt().into_tuple();

        require!(self.is_owner(&domain_name), "Not allowed {} {} {} {}", domain_name, token_id, token_nonce, _amount);

        let domain_record_exists = !self.domain_name(&domain_name).is_empty();
        require!(domain_record_exists, "Domain not exist");

        let mut domain = self.domain_name(&domain_name).get();
        if let OptionalValue::Some(_profile) = profile {
            domain.profile = Some(_profile);
        }
        if let OptionalValue::Some(_social) = social_media {
            domain.social_media = Some(_social);
        }
        if let OptionalValue::Some(_textrecord) = text_record {
            domain.text_record = Some(_textrecord);
        }
        if let OptionalValue::Some(_wallets) = wallets {
            domain.wallets = Some(_wallets);
        }

        self.domain_name(&domain_name).set(&domain);

        self.refund();
    }

    #[payable("ESDT")]
    #[endpoint]
    fn update_primary_address(
        &self,
        domain_name_or_sub_domain: ManagedBuffer,
        assign_to: OptionalValue<ManagedAddress>,
    ) {
        self.internal_update_primary_address(&domain_name_or_sub_domain, assign_to);
    }

    #[payable("*")]
    #[endpoint]
    fn register_sub_domain(&self, sub_domain: ManagedBuffer, address: ManagedAddress) {
        require!(self.is_owner(&sub_domain), "Not Allowed!");

        // let primary_domain = self.get_primary_domain(&sub_domain).unwrap();

        // let new_sub_domain = SubDomain {
        //     name: sub_domain.clone(),
        //     address: address.clone(),
        // };
        // let is_exist = self.sub_domains(&primary_domain).contains(&new_sub_domain);
        // require!(!is_exist, "Already registered");

        // self.get_egld_price_for_register_subdomain(sub_domain, address);

        self.refund();
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

    #[payable("ESDT")]
    #[endpoint]
    fn update_key_value(
        &self,
        domain_name: ManagedBuffer,
        key: ManagedBuffer,
        value: OptionalValue<ManagedBuffer>,
    ) {
        require!(self.is_owner(&domain_name), "Not Allowed!");

        match value {
            OptionalValue::Some(_value) => {
                self.resolve_domain_name_key(&domain_name, &key).set(_value)
            }
            OptionalValue::None => {
                self.resolve_domain_name_key(&domain_name, &key).clear();
            }
        }

        self.refund();
    }

    #[payable("*")]
    #[endpoint]
    fn transfer_domain(&self, domain_name: ManagedBuffer, new_owner: ManagedAddress) {
        let (token_id, token_nonce, _amount) = self.call_value().egld_or_single_esdt().into_tuple();
        let nft_token_id_mapper = self.domain_nft();
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
            &token_id == self.domain_nft().get_token_id_ref()
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

    #[payable("ESDT")]
    #[endpoint]
    fn remove_sub_domain(&self, sub_domain_name: ManagedBuffer, address: ManagedAddress) {
        require!(self.is_owner(&sub_domain_name), "Not Allowed!");

        let primary_domain = self.get_primary_domain(&sub_domain_name).unwrap();
        let mut sub_domain_mapper = self.sub_domains(&primary_domain);

        require!(
            sub_domain_mapper.contains(&SubDomain {
                name: sub_domain_name.clone(),
                address: address.clone()
            }),
            "there is no sub domain to remove"
        );

        sub_domain_mapper.swap_remove(&SubDomain {
            name: sub_domain_name,
            address,
        });

        self.refund();
    }

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
        let mut fees = self.rental_fee().get();
        match domain_length {
            1 => fees.one_letter = yearly_rent_usd,
            2 => fees.two_letter = yearly_rent_usd,
            3 => fees.three_letter = yearly_rent_usd,
            4 => fees.four_letter = yearly_rent_usd,
            _ => fees.other = yearly_rent_usd,
        }
        self.rental_fee().set(fees);
    }

    #[only_owner]
    #[endpoint]
    fn fetch_egld_usd_prices(&self){
        // let res = self.sync_get_equivalent(
        //     self.oracle_address().get(),
        //     TokenIdentifier::from_esdt_bytes(WEGLD_ID),
        //     BigUint::from(1_000_000_000_000_000_000u64),
        // );
        // self.egld_usd_price().set(res.to_u64().unwrap());
        // res
        self.internal_fetch_egld_usd_prices();
    }

    #[view]
    fn get_domain_nft_id(&self) -> TokenIdentifier {
        self.domain_nft().get_token_id_ref().clone()
    }
}
