# 44 Milady threat model

## Assets at risk

- lender USDG;
- borrower collateral;
- protocol reserves;
- insurance reserves;
- upgrade/admin authority.

## Oracle manipulation / stale data

Controls:

- Pyth Receiver owner check;
- PriceUpdateV2 discriminator check;
- full verification level;
- exact feed ID per market;
- maximum price age;
- positive-price check;
- confidence-band limit.

The keeper cannot bypass these checks.

## Unauthorized custody movement

Controls:

- collateral vaults are PDAs scoped to credit account + market;
- liquidity vault authority is the lending-pool PDA;
- all transfers use checked token transfers;
- borrower withdrawals require the owner signer;
- liquidation transfers only after fresh health failure.

## Insolvency

Controls:

- borrow LTV lower than liquidation threshold;
- utilization-based pricing;
- partial liquidation close factor;
- market-specific liquidation bonus;
- protocol reserve;
- dedicated insurance reserve;
- explicit bad-debt counter;
- recapitalization requires real USDG.

Bad debt is never silently erased from net assets.

## Interest rounding

A fractional numerator remainder is carried between accruals so frequent calls cannot suppress sub-micro-unit interest.

## Admin compromise

Controls:

- emergency authority separated from protocol authority;
- two-step protocol-authority rotation;
- two-step emergency-authority rotation;
- admin actions emit events;
- borrow and liquidation have independent switches;
- authority can be placed behind a multisig execution system.

The current code does not implement a time delay. A timelock is a mainnet-governance consideration.

## Keeper failure

The keeper is permissionless automation only. It may fail, stop, race another keeper or observe stale cached health. On-chain liquidation recomputes all critical conditions.

## Denial of service

Known surfaces:

- Pyth update availability;
- Solana congestion;
- maximum 8 collateral assets per credit account;
- transaction account count for multi-collateral health.

Mitigations include bounded collateral count, configurable oracle age and the ability to keep liquidation operational during a general protocol pause.

## Mock asset risk

Devnet USDG/NVDAx/AAPLx/SPYx/TSLAx are protocol-controlled test mints and must never be represented as real securities, stablecoins or redeemable assets.

## Remaining pre-mainnet work

- external audit;
- economic simulation with realistic correlated price shocks;
- governance/timelock design;
- production oracle/provider redundancy;
- incident runbooks;
- mainnet-specific caps;
- legal/compliance review for any tokenized real-world asset integration.
