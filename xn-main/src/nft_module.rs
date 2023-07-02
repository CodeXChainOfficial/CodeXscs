multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::data_module::DomainNameAttributes;

const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000; // 100%

#[multiversx_sc::module]
pub trait NftModule: 
    crate::callback_module::CallbackModule 
    + crate::storage_module::StorageModule {
    #[allow(clippy::too_many_arguments)]
    fn create_nft_with_attributes<T: TopEncode>(
        &self,
        name: ManagedBuffer,
        royalties: BigUint,
        attributes: T,
        uri: ManagedBuffer,
        _selling_price: BigUint,
        _token_used_as_payment: EgldOrEsdtTokenIdentifier,
        _token_used_as_payment_nonce: u64,
    ) -> u64 {
        self.require_token_issued();
        require!(royalties <= ROYALTIES_MAX, "Royalties cannot exceed 100%");

        let domain_nft = self.domain_nft();
        let token_id = domain_nft.get_token_id_ref();

        let mut serialized_attributes = ManagedBuffer::new();
        if let core::result::Result::Err(err) = attributes.top_encode(&mut serialized_attributes) {
            sc_panic!("Attributes encode error: {}", err.message_bytes());
        }

        let attributes_sha256 = self.crypto().sha256(&serialized_attributes);
        let attributes_hash = attributes_sha256.as_managed_buffer();
        let uris = ManagedVec::from_single_item(uri);
        let nft_nonce = self.send().esdt_nft_create(
            &token_id,
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
        require!(!self.domain_nft().is_empty(), "Token not issued");
    }

    fn burn_nft(&self, nft_nonce: u64) {
        let domain_nft = self.domain_nft();
        let token_id = domain_nft.get_token_id_ref();

        self.send()
            .esdt_local_burn(token_id, nft_nonce, &BigUint::from(NFT_AMOUNT));
    }

    fn mint_nft(
        &self,
        new_owner: &ManagedAddress,
        domain_name: &ManagedBuffer,
        selling_price: &BigUint,
        attributes: &DomainNameAttributes,
    ) -> u64 {
        let domain_nft = self.domain_nft();
        let token_id = domain_nft.get_token_id_ref();
        let name = domain_name.clone();
        let royalties = BigUint::from(self.royalties().get());
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
            token_id,
            nft_nonce,
            &BigUint::from(NFT_AMOUNT),
        );

        nft_nonce
    }

    fn is_owner_of_nft(
        &self,
        owner: &ManagedAddress,
        nft_nonce: u64
    ) -> bool {
        self.require_token_issued();
        let domain_nft = self.domain_nft();
        let token_id = domain_nft.get_token_id_ref();

        let balance = self.blockchain().get_esdt_balance(
            &owner,
            token_id,
            nft_nonce
        );

        balance == 1
    }   
}
