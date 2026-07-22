# AniPay Direct

On-chain escrow that pays rice farmers the instant their harvest is confirmed received — no more 30–45 day waits for a trader's payment.

## Problem

Nena, a smallholder rice farmer in Nueva Ecija, Philippines, sells her harvest to a trader who takes 30–45 days to pay after pickup. To cover seed and fertilizer for the next planting cycle in the meantime, she borrows from an informal lender at 10–20% monthly interest — a cost that eats directly into her already thin margins.

## Solution

At pickup, the trader deposits USDC into a Soroban escrow contract for that specific harvest lot. When the cooperative's warehouse officer confirms the delivery on-chain, the contract releases the USDC to the farmer's wallet within seconds. Stellar is essential here: settlement is near-instant and fees are near-zero, so even a modest harvest payment isn't eroded by transaction costs, and the escrow guarantees the trader can't walk away without paying.

## Timeline

Built for a hackathon/bootcamp timeframe — designed to be demoable end-to-end in under 2 minutes with two wallets and a single contract deployment.

## Stellar Features Used

- USDC transfers
- Soroban smart contracts (escrow + release logic)
- Trustlines (farmer and trader USDC trustlines)

## Vision and Purpose

To remove payment-timing risk as a source of predatory debt for smallholder farmers, by making the moment of physical delivery and the moment of payment the same moment — enforced by code, not by trust in the trader.

## Prerequisites

- Rust (1.79+ recommended)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Soroban CLI (v21.x): `cargo install --locked soroban-cli`

## How to Build

```bash
soroban contract build
```

The compiled Wasm will be at `target/wasm32-unknown-unknown/release/anipay_direct.wasm`.

## How to Test

```bash
cargo test
```

Runs all 5 tests: the happy-path escrow flow, a zero-amount rejection, a state-verification check, a duplicate-escrow-id rejection, and a double-release rejection.

## How to Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/anipay_direct.wasm \
  --source <YOUR_IDENTITY> \
  --network testnet
```

## Sample CLI Invocation

Setting the cooperative validator:

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_IDENTITY> \
  --network testnet \
  -- set_validator --validator <VALIDATOR_ADDRESS>
```

Creating an escrow for a harvest lot:

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <TRADER_IDENTITY> \
  --network testnet \
  -- create_escrow \
  --escrow_id 1 \
  --buyer <TRADER_ADDRESS> \
  --farmer <FARMER_ADDRESS> \
  --token <USDC_TOKEN_CONTRACT_ID> \
  --amount 5000000
```

Confirming delivery and releasing funds:

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <VALIDATOR_IDENTITY> \
  --network testnet \
  -- confirm_delivery --escrow_id 1
```

## License

MIT
