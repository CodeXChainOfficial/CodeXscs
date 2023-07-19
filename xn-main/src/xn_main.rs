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
pub mod utils_module;

use constant_module::{MIGRATION_PERIOD, NFT_AMOUNT};
use data_module::{
    Domain, DomainNameAttributes, PeriodType, Profile, RentalFee, Reservation, SocialMedia,
    TextRecord, Wallets
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
            three_letter: 10_000u64,
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

    #[payable("*")]
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
    fn update_domain_overview(
        &self,
        domain_name: ManagedBuffer,
        args: MultiValue5<
            ManagedBuffer,
            ManagedBuffer,
            ManagedBuffer,
            ManagedBuffer,
            ManagedBuffer,
        >,
    ) {
        require!(self.is_owner(&domain_name), "Not allowed");

        let domain_record_exists = !self.domain(&domain_name).is_empty();
        require!(domain_record_exists, "Domain not exist");

        let mut domain = self.domain(&domain_name).get();

        let (name, avatar, location, website, shortbio) = args.into_tuple();
        domain.profile = Some(Profile {
            name,
            avatar,
            location,
            website,
            shortbio,
        });
        self.domain(&domain_name).set(&domain);

        self.refund();
    }

    #[payable("*")]
    #[endpoint]
    fn update_domain_socials(
        &self,
        domain_name: ManagedBuffer,
        args: MultiValue6<
            ManagedBuffer,
            ManagedBuffer,
            ManagedBuffer,
            ManagedBuffer,
            ManagedBuffer,
            ManagedBuffer,
        >,
    ) {
        require!(self.is_owner(&domain_name), "Not allowed");

        let domain_record_exists = !self.domain(&domain_name).is_empty();
        require!(domain_record_exists, "Domain not exist");

        let mut domain = self.domain(&domain_name).get();

        let (telegram, discord, twitter, medium, facebook, other_link) = args.into_tuple();
        domain.social_media = Some(SocialMedia {
            telegram,
            discord,
            twitter,
            medium,
            facebook,
            other_link,
        });
        self.domain(&domain_name).set(&domain);

        self.refund();
    }

    #[payable("*")]
    #[endpoint]
    fn update_domain_wallets(
        &self,
        domain_name: ManagedBuffer,
        args: MultiValue3<ManagedBuffer, ManagedBuffer, ManagedBuffer>,
    ) {
        require!(self.is_owner(&domain_name), "Not allowed");

        let domain_record_exists = !self.domain(&domain_name).is_empty();
        require!(domain_record_exists, "Domain not exist");

        let mut domain = self.domain(&domain_name).get();
        let (egld, btc, eth) = args.into_tuple();
        domain.wallets = Some(Wallets { egld, btc, eth });
        self.domain(&domain_name).set(&domain);

        self.refund();
    }

    #[payable("*")]
    #[endpoint]
    fn update_domain_textrecord(
        &self,
        domain_name: ManagedBuffer,
        text_record: MultiValueManagedVec<TextRecord<Self::Api>>,
    ) {
        require!(self.is_owner(&domain_name), "Not allowed",);

        let domain_record_exists = !self.domain(&domain_name).is_empty();
        require!(domain_record_exists, "Domain not exist");

        let mut domain = self.domain(&domain_name).get();
        
        let mut records = ManagedVec::new();
        for record in text_record.iter() {
            records.push(record);
        }
        domain.text_record = Some(records);
        self.domain(&domain_name).set(&domain);

        self.refund();
    }

    #[payable("*")]
    #[endpoint]
    fn update_primary_address(
        &self,
        domain_name: ManagedBuffer,
    ) {
        let caller = self.blockchain().get_caller();
        require!(self.is_owner(&domain_name), "Not allowed");

        self.main_domain(&caller).set(domain_name);

        self.refund();
    }

    #[payable("*")]
    #[endpoint]
    fn register_sub_domain(&self, sub_domain: ManagedBuffer, address: ManagedAddress) {
        require!(self.is_owner(&sub_domain), "Not Allowed!");

        let primary_domain = self.get_primary_domain(&sub_domain).unwrap();

        require!(!self.sub_domains(&primary_domain).contains_key(&sub_domain.clone()), "Already registered");

        self.get_egld_price_for_register_subdomain(sub_domain, address);

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
        let domain_record_exists = !self.domain(&new_domain_name).is_empty();
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
        let new_domain_record = Domain {
            name: new_domain_name.clone(),
            expires_at: attributes.expires_at,
            nft_nonce,
            profile: Option::None,
            social_media: Option::None,
            wallets: Option::None,
            text_record: Option::None,
        };

        self.domain(&new_domain_name).set(new_domain_record.clone());
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
        require!(self.is_owner(&domain_name), "Not Allowed!");

        let (token_id, token_nonce, _amount) = self.call_value().egld_or_single_esdt().into_tuple();

        self.send().direct_esdt(
            &new_owner,
            &token_id.unwrap_esdt(),
            token_nonce,
            &BigUint::from(NFT_AMOUNT),
        );
        self.sub_domains(&domain_name).clear();
    }

    #[payable("ESDT")]
    #[endpoint]
    fn remove_sub_domain(&self, sub_domain_name: ManagedBuffer) {
        require!(self.is_owner(&sub_domain_name), "Not Allowed!");

        let primary_domain = self.get_primary_domain(&sub_domain_name).unwrap();

        require!(
            self.sub_domains(&primary_domain).contains_key(&sub_domain_name),
            "there is no sub domain to remove"
        );

        self.sub_domains(&primary_domain).remove(&sub_domain_name);
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
    fn fetch_egld_usd_prices(&self) {
        self.internal_fetch_egld_usd_prices();
    }

    #[view]
    fn get_domain_nft_id(&self) -> TokenIdentifier {
        self.domain_nft().get_token_id_ref().clone()
    }
}
