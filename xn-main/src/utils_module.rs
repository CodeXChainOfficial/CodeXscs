multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::constant_module::NFT_AMOUNT;
use crate::constant_module::{GRACE_PERIOD, MAX_LENGTH, MIN_LENGTH, YEAR_IN_SECONDS};
use crate::user_builtin;

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

#[multiversx_sc::module]
pub trait UtilsModule: crate::storage_module::StorageModule {
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

        let _name_str = match String::from_utf8(Vec::from(name_slice)) {
            Result::Ok(s) => s,
            Result::Err(_) => return Result::Err("name is not valid UTF-8"),
        };
        Result::Ok(())
    }

    fn split_domain_name(&self, name: &ManagedBuffer) -> ManagedVec<ManagedBuffer> {
        let mut parts = ManagedVec::new();
        let mut start: usize = 0;
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

    fn rent_price(&self, domain_name: &ManagedBuffer, secs: &u64) -> BigUint<Self::Api> {
        let rental_fee = self.rental_fee().get();

        let annual_price_usd = match domain_name.len() {
            1 => rental_fee.one_letter,
            2 => rental_fee.two_letter,
            3 => rental_fee.three_letter,
            4 => rental_fee.four_letter,
            _ => rental_fee.other,
        };

        let egld_usd_price = self.egld_usd_price().get();

        let price_egld = BigUint::from(annual_price_usd * u64::from(secs.clone()))
            * BigUint::from(egld_usd_price)
            / BigUint::from(YEAR_IN_SECONDS);

        price_egld
    }

    fn get_current_time(&self) -> u64 {
        self.blockchain().get_block_timestamp()
    }

    fn can_claim(&self, address: &ManagedAddress, domain_name: &ManagedBuffer) -> bool {
        if self.is_owner(domain_name) {
            return true;
        }

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

        if self.domain_name(&domain_name).is_empty() {
            return true;
        } else if self.get_current_time()
            >= self.domain_name(&domain_name).get().expires_at + GRACE_PERIOD
        {
            return true;
        }

        return false;
    }

    fn get_primary_domain(&self, domain_name: &ManagedBuffer) -> Option<ManagedBuffer> {
        let parts = self.split_domain_name(&domain_name);
        require!(parts.len() >= 2, "domain name is not valid");

        let domain_size = parts.get(parts.len() - 2).len() + 1 + parts.get(parts.len() - 1).len();

        let name = domain_name.copy_slice(domain_name.len() - domain_size, domain_size);
        name
    }

    fn is_owner(&self, domain_name: &ManagedBuffer) -> bool {
        let primary_domain = self.get_primary_domain(domain_name).unwrap();

        let payments = self.call_value().all_esdt_transfers();

        for payment in payments.iter() {
            if &payment.token_identifier == self.domain_nft().get_token_id_ref()
                && payment.token_nonce == self.domain_name(&primary_domain).get().nft_nonce
            {
                return true;
            }
        }
        return false;
    }

    fn refund(&self) {
        let caller = self.blockchain().get_caller();
        let payments = self.call_value().all_esdt_transfers();
        self.refund_with_detail(caller, payments);
    }

    fn refund_with_detail(&self, caller: ManagedAddress, payments: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>>) {
        let mut refund_payments = ManagedVec::new();
        for payment in payments.iter() {
            if payment.token_identifier != EgldOrEsdtTokenIdentifier::egld() {
                refund_payments.push(payment);
            }
        }

        self.send()
            .direct_multi(&caller, &refund_payments);
    }

    fn internal_update_primary_address(
        &self,
        domain_name: &ManagedBuffer,
        assign_to: OptionalValue<ManagedAddress>,
    ) {
        let caller = self.blockchain().get_caller();
        require!(self.is_owner(domain_name), "Not allowed");

        let domain_record = self.domain_name(&domain_name).get();

        let domain_nft = self.domain_nft();
        let token_id = domain_nft.get_token_id_ref();

        match assign_to {
            OptionalValue::Some(address) => {
                if address != caller {
                    self.send().direct_esdt(
                        &address,
                        token_id,
                        domain_record.nft_nonce,
                        &BigUint::from(NFT_AMOUNT),
                    );
                }
            }
            OptionalValue::None => {
                self.refund();
            }
        }
    }

    // fn internal_set_resolve_domain(&self, domain_name: &ManagedBuffer, address: &ManagedAddress) {
    //     self.user_builtin_proxy(address.clone())
    //         .set_user_name(domain_name.clone())
    //         .async_call()
    //         .with_callback(
    //             self.callbacks()
    //                 .set_user_name_callback(&domain_name, &address),
    //         )
    //         .call_and_exit();
    // }

    #[proxy]
    fn user_builtin_proxy(&self, to: ManagedAddress) -> user_builtin::Proxy<Self::Api>;
}
