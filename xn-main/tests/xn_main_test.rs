use xn_main::*;
use multiversx_sc::{
  types::{Address},
  codec::{Empty, multi_types::OptionalValue},
};

use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_buffer, managed_token_id, rust_biguint, testing_framework::*,
    DebugApi,
};

const WASM_PATH: &str = "output/xn-main.wasm";
const TOKEN_NAME: &[u8] = b"XN-123456";
const TOKEN_TICKER: &[u8] = b"XN-TICKER";
const MAIN_DOMAIN: &[u8] = b"first.mvx";
const SUB_MAIN_DOMAIN: &[u8] = b"a.first.mvx";
const YEAR_IN_SECONDS: u64 = 365 * 24 * 60 * 60;
const USD_TO_EGLD: u64 = 268000000000000;

struct ContractSetup<ContractObjBuilder>
where
  ContractObjBuilder: 'static + Copy + Fn() -> xn_main::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub oracle_address: Address,
    pub first_user_address: Address,
    pub second_user_address: Address,
    pub contract_wrapper:
        ContractObjWrapper<xn_main::ContractObj<DebugApi>, ContractObjBuilder>,
}

fn setup_contract<ContractObjBuilder>(
  contract_builder: ContractObjBuilder,
) -> ContractSetup<ContractObjBuilder>
where
  ContractObjBuilder: 'static + Copy + Fn() -> xn_main::ContractObj<DebugApi>,
{
  let rust_zero = rust_biguint!(0u64);
  let mut blockchain_wrapper = BlockchainStateWrapper::new();
  let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
  let oracle_address = blockchain_wrapper.create_user_account(&rust_zero);
  let first_user_address = blockchain_wrapper.create_user_account(&rust_zero);
  let second_user_address = blockchain_wrapper.create_user_account(&rust_zero);
  let contract_wrapper = blockchain_wrapper.create_sc_account(
      &rust_zero,
      Some(&owner_address),
      contract_builder,
      WASM_PATH,
  );
  let DEFAULT_USD_TO_EGLD = rust_biguint!(USD_TO_EGLD);

  blockchain_wrapper.set_egld_balance(&owner_address, &(rust_biguint!(100_000_000) * DEFAULT_USD_TO_EGLD.clone()));
  blockchain_wrapper.set_egld_balance(&first_user_address, &(rust_biguint!(100_000_000) * DEFAULT_USD_TO_EGLD.clone()));
  blockchain_wrapper.set_egld_balance(&second_user_address, &(rust_biguint!(100_000_000) * DEFAULT_USD_TO_EGLD.clone()));

  blockchain_wrapper
    .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
      let token_name = managed_buffer!(TOKEN_NAME);
      let token_ticker = managed_buffer!(TOKEN_TICKER);
      let managed_oracle_address = managed_address!(&oracle_address);

      // call init to initlaize smart contract
      sc.init(
        managed_oracle_address,
        token_name.clone(),
        token_ticker.clone(),
      );

    })
    .assert_ok();

  blockchain_wrapper.add_mandos_set_account(contract_wrapper.address_ref());

  ContractSetup {
    blockchain_wrapper,
    owner_address,
    oracle_address,
    first_user_address,
    second_user_address,
    contract_wrapper,
  }
}
/*
test init and generate scenario for it
*/
#[test]
fn init_test() {
  let contract_setup = setup_contract(xn_main::contract_obj);

  contract_setup
    .blockchain_wrapper
    .write_mandos_output("_generated_init.scen.json");
}

/*
test for owner to issue token
*/
#[test]
fn issue_token_test() {
  let mut contract_setup = setup_contract(xn_main::contract_obj);
  let b_wrapper = &mut contract_setup.blockchain_wrapper;
  let owner_addr = &contract_setup.owner_address;

  b_wrapper
    .execute_tx(&owner_addr, &contract_setup.contract_wrapper, &(rust_biguint!(100_000)), |sc| {
      let token_name = managed_buffer!(TOKEN_NAME);
      let token_ticker = managed_buffer!(TOKEN_TICKER);
      sc.issue_token(
        token_name.clone(),
        token_ticker.clone()
      );
      let is_empty = sc.nft_token_id().is_empty();
      // check if NFT issued
      assert_eq!(is_empty, false);
    }
  )
  .assert_ok();

}

/*
test for register, renew or assign domain
*/
#[test]
fn register_or_renew_test() {
  let mut contract_setup = setup_contract(xn_main::contract_obj);
  let b_wrapper = &mut contract_setup.blockchain_wrapper;
  let user_addr = &contract_setup.first_user_address;
  let second_user_addr = &contract_setup.second_user_address;
  let DEFAULT_USD_TO_EGLD = rust_biguint!(USD_TO_EGLD);
  let block_timestamp: u64 = 1000;
  let mut global_token_id: &[u8] = &[];
  let mut global_nounce: u64 = 0;
  
  b_wrapper.set_block_timestamp(block_timestamp);

  b_wrapper
    .execute_tx(
      user_addr,
      &contract_setup.contract_wrapper,
      &(rust_biguint!(100_000) * DEFAULT_USD_TO_EGLD.clone()),
      |sc| {
        let domain_name = managed_buffer!(b"first.mvx");
        let years: u8 = 1;
        let token_name = managed_buffer!(TOKEN_NAME);
        let token_ticker = managed_buffer!(TOKEN_TICKER);
        let managed_second_user = managed_address!(second_user_addr);
        let mut is_empty = sc.nft_token_id().is_empty();
        assert_eq!(is_empty, true);
        sc.issue_token(
          token_name.clone(),
          token_ticker.clone()
        );
        is_empty = sc.nft_token_id().is_empty();
        assert_eq!(is_empty, false);
        let token_id = sc.nft_token_id().get();
        // global_token_id = token_id.to_boxed_bytes().as_slice();
        sc.register_or_renew(
          domain_name.clone(),
          years,
          None.into()
        );

        let empty_domain_record = sc.domain_name(&domain_name).is_empty();
        // check if domain_record with domain_name exists
        assert_eq!(empty_domain_record, false);
        let domain_record = sc.domain_name(&domain_name).get();
        // check saved values of domain_record with inputs
        assert_eq!(domain_record.name.parse_as_u64().unwrap(), domain_name.parse_as_u64().unwrap());
        assert_eq!(domain_record.expires_at, block_timestamp + years as u64 * YEAR_IN_SECONDS);
        global_nounce = domain_record.nft_nonce.clone();
        let mut domain_owner = sc.owner_domain_name(&domain_name).get();
        // check if domain owner is first user
        assert_eq!(domain_owner.to_address(), *user_addr);

        sc.register_or_renew(
          domain_name.clone(),
          years,
          OptionalValue::Some(managed_second_user)
        );
        domain_owner = sc.owner_domain_name(&domain_name).get();
        // check if domain owner changed to second user
        assert_eq!(domain_owner.to_address(), *second_user_addr);
      },
    )
    .assert_ok();
  
  // check NFT balance
  // b_wrapper.check_nft_balance::<Empty>(
  //   user_addr,
  //   global_token_id,
  //   global_nounce,
  //   &rust_biguint!(1),
  //   None,
  // );
}

/*
test for request, and accept request
*/

#[test]
fn accpet_request() {
  let mut contract_setup = setup_contract(xn_main::contract_obj);
  let b_wrapper = &mut contract_setup.blockchain_wrapper;
  let DEFAULT_USD_TO_EGLD = rust_biguint!(USD_TO_EGLD);
  let user_addr = &contract_setup.first_user_address;
  let second_user_addr = &contract_setup.second_user_address;
  let rust_zero = rust_biguint!(0u64);
  
  b_wrapper
  .execute_tx(
    user_addr,
    &contract_setup.contract_wrapper,
    &(rust_biguint!(100_000) * DEFAULT_USD_TO_EGLD.clone()),
    |sc| {
        let domain_name = managed_buffer!(MAIN_DOMAIN);
        let years: u8 = 1;
        let token_name = managed_buffer!(TOKEN_NAME);
        let token_ticker = managed_buffer!(TOKEN_TICKER);
        let managed_first_user = managed_address!(user_addr);
        let managed_second_user = managed_address!(second_user_addr);
        let mut is_empty = sc.nft_token_id().is_empty();
        assert_eq!(is_empty, true);
        sc.issue_token(
          token_name.clone(),
          token_ticker.clone()
        );
        is_empty = sc.nft_token_id().is_empty();
        assert_eq!(is_empty, false);
        sc.register_or_renew(
          domain_name.clone(),
          years,
          OptionalValue::Some(managed_first_user)
        );

        sc.update_primary_address(
          domain_name.clone(), 
          OptionalValue::Some(managed_second_user)
        );
        let domain_owner = sc.owner_domain_name(&domain_name.clone()).get();
        // check if domain owner changed to second user
        assert_eq!(domain_owner.to_address(), *second_user_addr);
        let requested_address = sc.accept_request(&domain_name).get();
        // check if domain requested to second user
        assert_eq!(requested_address.to_address(), *second_user_addr);
        let resolve_domain_address = sc.resolve_domain_name(&domain_name).get();
        // check if resolve address of domain is still first user
        assert_eq!(resolve_domain_address.to_address(), *user_addr);
      }
    )
    .assert_ok();

  // accepet 
  b_wrapper
    .execute_tx(
      second_user_addr,
      &contract_setup.contract_wrapper,
      &rust_zero,
      |sc| {
        let domain_name = managed_buffer!(b"first.mvx");
        // check if domain requested to second user
        sc.accept(
          domain_name.clone(), 
        );
        let request_is_empty = sc.accept_request(&domain_name.clone()).is_empty();
        // check if accept request cleared
        assert_eq!(request_is_empty, true);
        let resolve_domain_address = sc.resolve_domain_name(&domain_name).get();
        // // check if resolve domain name cleared
        assert_eq!(resolve_domain_address.to_address(), *second_user_addr);
      }
    )
    .assert_ok();
}

/*
test for request, and revoke request
*/
#[test]
fn revoke_request() {
  let mut contract_setup = setup_contract(xn_main::contract_obj);
  let b_wrapper = &mut contract_setup.blockchain_wrapper;
  let DEFAULT_USD_TO_EGLD = rust_biguint!(USD_TO_EGLD);
  let user_addr = &contract_setup.first_user_address;
  let second_user_addr = &contract_setup.second_user_address;
  let rust_zero = rust_biguint!(0u64);
  
  b_wrapper
  .execute_tx(
    user_addr,
    &contract_setup.contract_wrapper,
    &(rust_biguint!(100_000) * DEFAULT_USD_TO_EGLD.clone()),
    |sc| {
        let domain_name = managed_buffer!(MAIN_DOMAIN);
        let years: u8 = 1;
        let token_name = managed_buffer!(TOKEN_NAME);
        let token_ticker = managed_buffer!(TOKEN_TICKER);
        let managed_first_user = managed_address!(user_addr);
        let managed_second_user = managed_address!(second_user_addr);
        let mut is_empty = sc.nft_token_id().is_empty();
        assert_eq!(is_empty, true);
        sc.issue_token(
          token_name.clone(),
          token_ticker.clone()
        );
        is_empty = sc.nft_token_id().is_empty();
        assert_eq!(is_empty, false);
        sc.register_or_renew(
          domain_name.clone(),
          years,
          OptionalValue::Some(managed_first_user)
        );

        sc.update_primary_address(
          domain_name.clone(), 
          OptionalValue::Some(managed_second_user)
        );
        let domain_owner = sc.owner_domain_name(&domain_name.clone()).get();
        // check if domain owner changed to second user
        assert_eq!(domain_owner.to_address(), *second_user_addr);
        let requested_address = sc.accept_request(&domain_name).get();
        // check if domain requested to second user
        assert_eq!(requested_address.to_address(), *second_user_addr);
        let resolve_domain_address = sc.resolve_domain_name(&domain_name).get();
        // check if resolve address of domain is still first user
        assert_eq!(resolve_domain_address.to_address(), *user_addr);
      }
    )
    .assert_ok();

  // revoke 
  b_wrapper
    .execute_tx(
      second_user_addr,
      &contract_setup.contract_wrapper,
      &rust_zero,
      |sc| {
        let domain_name = managed_buffer!(b"first.mvx");
        // check if domain requested to second user
        sc.revoke_accept_request(
          domain_name.clone(), 
        );
        let request_is_empty = sc.accept_request(&domain_name.clone()).is_empty();
        // check if accept request cleared
        assert_eq!(request_is_empty, true);
      }
    )
    .assert_ok();
}


/*
test for creating subdomain
*/
#[test]
fn create_sub_domain() {
  let mut contract_setup = setup_contract(xn_main::contract_obj);
  let b_wrapper = &mut contract_setup.blockchain_wrapper;
  let DEFAULT_USD_TO_EGLD = rust_biguint!(USD_TO_EGLD);
  let user_addr = &contract_setup.first_user_address;
  let second_user_addr = &contract_setup.second_user_address;
  let rust_zero = rust_biguint!(0u64);
  
  b_wrapper
  .execute_tx(
    user_addr,
    &contract_setup.contract_wrapper,
    &(rust_biguint!(100_000) * DEFAULT_USD_TO_EGLD.clone()),
    |sc| {
        let domain_name = managed_buffer!(MAIN_DOMAIN);
        let sub_domain_name = managed_buffer!(SUB_MAIN_DOMAIN);
        let years: u8 = 1;
        let token_name = managed_buffer!(TOKEN_NAME);
        let token_ticker = managed_buffer!(TOKEN_TICKER);
        let managed_first_user = managed_address!(user_addr);
        let managed_second_user = managed_address!(second_user_addr);
        let mut is_empty = sc.nft_token_id().is_empty();
        assert_eq!(is_empty, true);
        sc.issue_token(
          token_name.clone(),
          token_ticker.clone()
        );
        is_empty = sc.nft_token_id().is_empty();
        assert_eq!(is_empty, false);
        sc.register_or_renew(
          domain_name.clone(),
          years,
          OptionalValue::Some(managed_first_user)
        );

        sc.update_primary_address(
          sub_domain_name.clone(), 
          OptionalValue::Some(managed_second_user)
        );
        let domain_owner = sc.owner_domain_name(&sub_domain_name.clone()).get();
        // check if domain owner changed to second user
        assert_eq!(domain_owner.to_address(), *second_user_addr);
        let requested_address = sc.accept_request(&sub_domain_name).get();
        // check if domain requested to second user
        assert_eq!(requested_address.to_address(), *second_user_addr);
        let resolve_domain_address = sc.resolve_domain_name(&sub_domain_name).get();
        // check if resolve address of domain is still first user
        assert_eq!(resolve_domain_address.to_address(), *user_addr);
      }
    )
    .assert_ok();
}
