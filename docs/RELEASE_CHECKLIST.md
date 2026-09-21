# 44 Milady release checklist

## Source
- [ ] CI green
- [ ] Rust formatting green
- [ ] Rust unit tests green
- [ ] model + adversarial tests green
- [ ] keeper tests green
- [ ] Next.js production build green
- [ ] Anchor SBF build green

## Identity
- [ ] stable program keypair generated offline/securely
- [ ] keypair backed up
- [ ] `anchor keys sync` run
- [ ] `declare_id!` equals program keypair pubkey
- [ ] `Anchor.toml` equals program keypair pubkey
- [ ] no private key material committed

## Devnet deployer
- [ ] deployer key stored outside repo
- [ ] deployer funded with sufficient Devnet SOL
- [ ] Solana CLI points to Devnet

## Oracle configuration
- [ ] feed IDs verified against current Pyth documentation/API
- [ ] price-feed accounts verified on Solana Devnet
- [ ] freshness limits appropriate for feed cadence
- [ ] confidence limits reviewed

## Bootstrap
- [ ] mock mints created
- [ ] protocol PDA owns mock mint authority
- [ ] faucet configs created
- [ ] markets created
- [ ] USDG lending pool created
- [ ] liquidation close factor checked
- [ ] borrowing remains disabled until verification

## Verification
- [ ] executable program found at canonical ID
- [ ] protocol version >= 7
- [ ] all mint/market addresses recorded
- [ ] live faucet claim succeeds
- [ ] live supply succeeds
- [ ] live Pyth health refresh succeeds
- [ ] live borrow succeeds
- [ ] live MAX repay succeeds
- [ ] final E2E debt = 0

## Operations
- [ ] monitor process configured
- [ ] keeper starts in dry-run
- [ ] keeper has dedicated liquidator wallet before live mode
- [ ] bad-debt alerting enabled
- [ ] authority/emergency roles documented
- [ ] authority rotation tested
- [ ] public deployment manifest published
