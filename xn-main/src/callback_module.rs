multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait CallbackModule: crate::storage_module::StorageModule {
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
}