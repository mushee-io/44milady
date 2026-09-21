# 44 Milady Security

44 Milady is experimental software. Devnet assets have no monetary value.

## Reporting

Do not publish a reproducible exploit against a live deployment before maintainers have had a reasonable opportunity to respond.

Include:

- affected instruction/account;
- network and program ID;
- transaction/signature when relevant;
- minimal reproduction;
- expected vs actual behavior;
- impact analysis.

## Security boundaries

The Solana program is the final authority for:

- signer and PDA validation;
- token mint/vault ownership;
- oracle owner/feed/freshness/confidence validation;
- LTV and liquidation health;
- close factor and liquidation bonus;
- interest/debt accounting;
- reserve/insurance/bad-debt accounting.

Frontend, keeper and monitoring data are never authoritative for solvency.

## Key handling

Program keypairs, deployer keypairs and mnemonics must never be committed. The repository contains only public addresses and configuration templates.

## Release policy

A public deployment is not considered released until:

1. CI passes;
2. Anchor SBF build passes;
3. canonical program ID is committed;
4. deployment verification passes against an executable Devnet program;
5. live E2E borrow/repay succeeds;
6. admin authority is rotated away from a disposable deployer when appropriate.
