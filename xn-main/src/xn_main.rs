#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const GRACE_PERIOD: u64 = 21 * 24 * 60 * 60; // 21 days
const YEAR_IN_SECONDS: u64 = 365 * 24 * 60 * 60; // 1 year (365 days)
const MIN_LENGTH: usize = 3;
const MAX_LENGTH: usize = 256;

// objects
#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct DomainName<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
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

/// A contract that allows anyone to send a fixed sum, locks it for a while and then allows users to take it back.
/// Sending funds to the contract is called "ping".
/// Taking the same funds back is called "pong".
///
/// Restrictions:
/// - Only the set amount can be `ping`-ed, no more, no less.
/// - `pong` can only be called after a certain period after `ping`.

#[multiversx_sc::contract]
pub trait XnMain {
    #[init]
    fn init(&self) {}

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
        require!(self.is_name_valid(&domain_name), "name is not valid");
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

        let new_domain_record = DomainName {
            name: domain_name.clone(),
            expires_at: since + (u64::from(years) * YEAR_IN_SECONDS),
        };

        self.domain_name(&domain_name)
            .set(new_domain_record.clone());
        self._update_primary_address(&domain_name, assign_to);

        // TODO: NFTs
        // if exist alredy, burn for old owner
        // mint NFT for new owner


        // return extra EGLD if customer sent more than required
        if price < payment {
            let excess = payment - price;
            self.send().direct(&caller, &token, 0, &excess);
        }
    }

    fn is_name_valid(&self, domain_name: &ManagedBuffer) -> bool {
        // require!(domain_name.len() <= MAX_LENGTH, "name too long");
        require!(domain_name.len() >= MIN_LENGTH, "name too short");

        // ends with a whitelisted TLD

        // TODO: validate the types of characters
        // punycode? https://chromium.googlesource.com/chromium/src/+/main/docs/idn.md

        true
    }

    fn rent_price(&self, domain_name: &ManagedBuffer, years: &u8) -> BigUint<Self::Api> {
        let len = domain_name.len();

        let mut yearly_price = 1;

        if len <= MIN_LENGTH {
            yearly_price = 100
        } else if len == MIN_LENGTH + 1 {
            yearly_price = 10
        }

        (self.Egld(yearly_price) * u64::from(years.clone())).into()
    }

    fn Egld(&self, price: u64) -> u64 {
        price * 10_000_000_000_000_000
    }

    fn get_current_time(&self) -> u64 {
        self.blockchain().get_block_timestamp()
    }

    fn can_claim(&self, address: &ManagedAddress, domain_name: &ManagedBuffer) -> bool {
            // Is not found
            self.domain_name(&domain_name).is_empty() 
            // Is out of grace period or not found
            || (self.domain_name(&domain_name).get().expires_at + GRACE_PERIOD < self.get_current_time())
            // if is owner
            || self.is_owner(&address, &domain_name)
    }

    fn is_owner(&self, address: &ManagedAddress, domain_name: &ManagedBuffer) -> bool {
        // has NFT
        // NFT is the key to ownership

        // is reserved for him
        if self.owner_domain_name(&address).get() == domain_name.clone() {
            return true;
        }

        require!(false, "Not Permitted! only owner can call this method");
        false
    }

    #[endpoint]
    fn is_empty(
        &self,
        domain_name: ManagedBuffer
    ) -> DomainName<Self::Api> {
        if(self.domain_name(&domain_name).is_empty()) {
            return self.domain_name(&domain_name).get();
        }
        else {
            require!(false, "not empty");
            return self.domain_name(&domain_name).get();
        }
        
    }
    #[endpoint]
    fn update_primary_address(
        &self,
        domain_name: ManagedBuffer,
        assign_to: OptionalValue<ManagedAddress>,
    ) {
        let caller = self.blockchain().get_caller();

        self.is_owner(&caller, &domain_name);

        self._update_primary_address(&domain_name, assign_to)
    }

    fn _update_primary_address(
        &self,
        domain_name: &ManagedBuffer,
        assign_to: OptionalValue<ManagedAddress>,
    ) {
        let caller = self.blockchain().get_caller();

        match assign_to {
            OptionalValue::Some(address) => {
                if address == caller {
                    self.resolve_domain_name(&domain_name).set(address)
                } else {
                    self.accept_request(&domain_name).set(address)
                }
            }
            OptionalValue::None => {
                self.resolve_domain_name(&domain_name).clear();
            }
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
        self.is_owner(&caller, &domain_name);

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

        self.resolve_domain_name(&domain_name).set(caller);
        self.accept_request(&domain_name).clear();
    }

    #[endpoint]
    fn revokeAcceptRequest(&self, domain_name: ManagedBuffer) {
        let caller = self.blockchain().get_caller();

        self.is_owner(&caller, &domain_name);

        self.accept_request(&domain_name).clear();
    }

    // endpoints - admin-only

    #[only_owner]
    #[endpoint]
    fn reserve(&self, reservations: ManagedVec<Reservation<Self::Api>>) {
        for reservation in reservations.iter() {
            let name = reservation.domain_name;
            let domain_name = DomainName {
                name: name.clone(),
                expires_at: reservation.until,
            };

            self.domain_name(&name).set_if_empty(domain_name);
            self.owner_domain_name(&reservation.reserved_for)
                .set_if_empty(name);
        }
    }

    #[only_owner]
    #[endpoint]
    fn set_prices(&self, address: ManagedAddress) {}

    // views

    // #[view]
    // fn did_user_ping(&self, address: &ManagedAddress) -> bool {
    //     // !self.user_ping_timestamp(address).is_empty()
    // }

    // #[view]
    // fn get_pong_enable_timestamp(&self, address: &ManagedAddress) -> u64 {
    //     if !self.did_user_ping(address) {
    //         return 0;
    //     }

    //     // let user_ping_timestamp = self.user_ping_timestamp(address).get();
    //     // let duration_in_seconds = self.duration_in_seconds().get();

    //     // user_ping_timestamp + duration_in_seconds
    // }

    // #[view]
    // fn get_time_to_pong(&self, address: &ManagedAddress) -> OptionalValue<u64> {
    //     if !self.did_user_ping(address) {
    //         return OptionalValue::None;
    //     }

    //     let pong_enable_timestamp = self.get_pong_enable_timestamp(address);
    //     let current_timestamp = self.blockchain().get_block_timestamp();

    //     if current_timestamp >= pong_enable_timestamp {
    //         OptionalValue::Some(0)
    //     } else {
    //         let time_left = pong_enable_timestamp - current_timestamp;
    //         OptionalValue::Some(time_left)
    //     }
    // }

    // storage
    #[view(get_accept_request)]
    #[storage_mapper("accept_request")]
    fn accept_request(&self, domain_name: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[view(get_domain_name)]
    #[storage_mapper("domain_name")]
    fn domain_name(&self, domain_name: &ManagedBuffer) -> SingleValueMapper<DomainName<Self::Api>>;

    #[view(get_owner_domain_name)]
    #[storage_mapper("owner_domain_name")]
    fn owner_domain_name(&self, address: &ManagedAddress) -> SingleValueMapper<ManagedBuffer>;

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
}
