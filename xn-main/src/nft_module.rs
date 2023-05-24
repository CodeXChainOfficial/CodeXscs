multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::data_module::DomainNameAttributes;

const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000; // 100%

#[multiversx_sc::module]
pub trait NftModule: 
    crate::callback_module::CallbackModule 
    + crate::storage_module::StorageModule {
    // private

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

    // fn get_nft_nonce_from_domain_name(&self, domain_name: &ManagedBuffer) -> u64 {
    //     // Compute a nonce based on the domain name.
    //     self.crypto().sha256(domain_name).into()
    // }

    fn burn_nft(&self, nft_nonce: u64) {
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
}
