use crate::{
    constant_module::{
        DAY_IN_SECONDS, HOUR_IN_SECONDS, MIN_IN_SECONDS, MONTH_IN_SECONDS, SUB_DOMAIN_COST_USD,
        WEGLD_ID, YEAR_IN_SECONDS,
    },
    data_module::{Domain, DomainNameAttributes, PeriodType, SubDomain},
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait CallbackModule:
    crate::storage_module::StorageModule
    + crate::nft_module::NftModule
    + crate::utils_module::UtilsModule
{
    #[callback]
    fn fetch_egld_usd_prices_callback(&self, #[call_result] result: ManagedAsyncCallResult<u64>) {
        match result {
            ManagedAsyncCallResult::Ok(price) => {
                self.egld_usd_price().set(price);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[callback]
    fn xexchange_callback(&self, #[call_result] result: ManagedAsyncCallResult<BigUint>) {
        match result {
            ManagedAsyncCallResult::Ok(amount_out) => {
                if amount_out.to_u64().is_some() {
                    self.egld_usd_price().set(amount_out.to_u64().unwrap());
                } else {
                    self.egld_usd_price().set(1);
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                self.egld_usd_price().set(11);
            }
        }
    }

    #[payable("*")]
    #[callback]
    fn register_or_renew_callback(
        &self,
        caller: ManagedAddress,
        payments: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>>,
        domain_name: ManagedBuffer,
        period: u8,
        unit: PeriodType,
        assign_to: Option<ManagedAddress>,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(amount_out) => {
                self.egld_usd_price().set(amount_out.to_u64().unwrap());

                let mut egld_amount = BigUint::zero();
                for payment in payments.iter() {
                    if payment.clone().token_identifier.into_managed_buffer()
                        == ManagedBuffer::from(WEGLD_ID)
                    {
                        egld_amount = payment.amount;
                        break;
                    }
                }

                let secs = [
                    YEAR_IN_SECONDS,
                    MONTH_IN_SECONDS,
                    DAY_IN_SECONDS,
                    HOUR_IN_SECONDS,
                    MIN_IN_SECONDS,
                ];
                let period_secs: u64 = u64::from(period) * secs[unit as usize];

                let price = self.rent_price(&domain_name, &period_secs);

                if price > egld_amount {
                    self.refund_all(caller, payments);
                    sc_panic!("Insufficient EGLD Funds {} {}", price.to_u64().unwrap(), egld_amount.to_u64().unwrap());
                }

                let since = self.blockchain().get_block_timestamp();

                if !self.domain(&domain_name).is_empty() {
                    let mut domain_record = self.domain(&domain_name).get();
                    domain_record.expires_at = since + period_secs;
                    self.domain(&domain_name).set(domain_record.clone());
                } else {
                    let attributes = DomainNameAttributes {
                        expires_at: since + period_secs,
                    };
                    let nft_nonce;
                    match assign_to {
                        Some(to) => {
                            nft_nonce = self.mint_nft(&to, &domain_name, &price, &attributes);
                        }
                        None => {
                            nft_nonce = self.mint_nft(&caller, &domain_name, &price, &attributes);
                        }
                    }
                    let new_domain_record = Domain {
                        name: domain_name.clone(),
                        expires_at: attributes.expires_at,
                        nft_nonce,
                        profile: Option::None,
                        social_media: Option::None,
                        wallets: Option::None,
                        text_record: Option::None,
                    };

                    self.domain(&domain_name)
                        .set(new_domain_record.clone());
                }

                if price < egld_amount {
                    let excess = egld_amount - price;
                    self.send().direct(
                        &caller,
                        &EgldOrEsdtTokenIdentifier::esdt(TokenIdentifier::from(WEGLD_ID)),
                        0,
                        &excess,
                    );
                }
                self.refund_with_payments(caller, payments);
            }
            ManagedAsyncCallResult::Err(_) => {
                
            }
        }
    }

    #[callback]
    fn register_subdomain_callback(
        &self,
        caller: ManagedAddress,
        payments: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>>,
        sub_domain: ManagedBuffer,
        address: ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(amount_out) => {
                self.egld_usd_price().set(amount_out.to_u64().unwrap());

                let mut egld_amount = BigUint::zero();

                for payment in payments.iter() {
                    if payment.clone().token_identifier.into_managed_buffer()
                        == ManagedBuffer::from(WEGLD_ID)
                    {
                        egld_amount = payment.amount;
                        break;
                    }
                }

                let egld_usd_price = self.egld_usd_price().get();
                let price = BigUint::from(SUB_DOMAIN_COST_USD) / BigUint::from(egld_usd_price);

                if price > egld_amount {
                    self.refund_all(caller, payments);
                    sc_panic!("Insufficient EGLD Funds {} {}", price.to_u64().unwrap(), egld_amount.to_u64().unwrap());
                }

                let primary_domain = self.get_primary_domain(&sub_domain).unwrap();

                let new_sub_domain = SubDomain {
                    name: sub_domain.clone(),
                    address,
                };

                let _ = &mut self.sub_domains(&primary_domain).insert(sub_domain, new_sub_domain);

                if price < egld_amount {
                    let excess = egld_amount - price;
                    self.send().direct(
                        &caller,
                        &EgldOrEsdtTokenIdentifier::esdt(TokenIdentifier::from(WEGLD_ID)),
                        0,
                        &excess,
                    );
                }

                self.refund_with_payments(caller, payments);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }
}
