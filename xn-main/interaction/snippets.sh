WALLET_PEM="./wallet-owner.pem"
WASM_PATH="output/xn-main.wasm"
PROXY=https://devnet-gateway.multiversx.com
CHAIN_ID=D

################################################
ADDRESS=$(mxpy data load --key=address-devnet)
################################################

PROXY_DEV_ADDRESS="erd1qqqqqqqqqqqqqpgqq67uv84ma3cekpa55l4l68ajzhq8qm3u0n4s20ecvx"
PROXY_MAIN_ADDRESS="erd1qqqqqqqqqqqqqpgqeel2kumf0r8ffyhth7pqdujjat9nx0862jpsg2pqaq"
PROXY_DEV_ADDRESS_HEX="0x$(mxpy wallet bech32 --decode ${PROXY_DEV_ADDRESS})"
deploy() {
    mxpy --verbose contract deploy \
    --project=${PROJECT} \
    --recall-nonce \
    --pem=${WALLET_PEM} \
    --gas-limit=600000000 \
    --arguments ${PROXY_DEV_ADDRESS_HEX} \
    --send \
    --metadata-payable \
    --outfile="deploy-devnet.interaction.json" \
    --proxy=${PROXY} \
    --chain=${CHAIN_ID} || return

    ADDRESS=$(mxpy data parse --file="deploy-devnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-devnet --value=${ADDRESS}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

issue_token() {

    local TOKEN_DISPLAY_NAME=0x4142434445  
    local TOKEN_TICKER=0x4142434445  

    mxpy --verbose contract call ${ADDRESS} \
    --recall-nonce --pem=${WALLET_PEM} \
    --gas-limit=500000000 --value=50000000000000000 \
    --function="issue_token" \
    --arguments ${TOKEN_DISPLAY_NAME} ${TOKEN_TICKER} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}


set_local_roles() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${WALLET_PEM} \
    --gas-limit=100000000 --function="set_local_roles" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

register_or_renew() {

    local DOMAIN_NAME=0x6f776e65722e6d7678  
    local PERID=1
    local UNIT=3  

    mxpy --verbose contract call ${ADDRESS} \
    --recall-nonce --pem=${WALLET_PEM} \
    --gas-limit=60000000 --value=50000000000000000 \
    --function="register_or_renew" \
    --arguments ${DOMAIN_NAME} ${PERID} ${UNIT} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

register_sub_domain() {

    local SUB_DOMAIN_NAME=0x737562646f6d61696e2e6f776e65722e6d7678  
    local ADDR="erd1n63y70nsmh6hr9dde9spz4tk2gf42vskhus54mkc0unp4ssjt95s5zw6g4"
    local ADDR_HEX="0x$(mxpy wallet bech32 --decode ${ADDR})"

    mxpy --verbose contract call ${ADDRESS} \
    --recall-nonce --pem=${WALLET_PEM} \
    --gas-limit=60000000 --value=50000000000000000 \
    --function="register_sub_domain" \
    --arguments ${SUB_DOMAIN_NAME} ${ADDR_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}


remove_sub_domain() {

    local SUB_DOMAIN_NAME=0x737562646f6d61696e2e6f776e65722e6d7678  

    mxpy --verbose contract call ${ADDRESS} \
    --recall-nonce --pem=${WALLET_PEM} \
    --gas-limit=60000000 --value=0 \
    --function="remove_sub_domain" \
    --arguments ${SUB_DOMAIN_NAME} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

transfer_domain() {

    local DOMAIN_NAME=0x737562646f6d61696e2e6f776e65722e6d7678  
    local ADDR="erd1qqqqqqqqqqqqqpgqn2wwdqw3q0828hwprkahl7s6nv80n899t95s004j8d"
    local ADDR_HEX="0x$(mxpy wallet bech32 --decode ${PROXY_DEV_ADDRESS})"

    mxpy --verbose contract call ${ADDRESS} \
    --recall-nonce --pem=${WALLET_PEM} \
    --gas-limit=60000000 --value=0 \
    --function="transfer_domain" \
    --arguments ${DOMAIN_NAME}  ${ADDR_HEX}\
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

getPrice() {

    mxpy contract query ${ADDRESS} \
    --function="get_egld_usd_price" \
    --proxy=${PROXY}
}

getSubDomains() {

    local DOMAIN_NAME=0x6f776e65722e6d7678 

    mxpy contract query ${ADDRESS} \
    --function="get_sub_domains" \
    --arguments ${DOMAIN_NAME} \
    --proxy=${PROXY}
}
