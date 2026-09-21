const markets = [
  { symbol: "NVDAx", ltv: "60%", liq: "75%", source: "Pyth" },
  { symbol: "AAPLx", ltv: "60%", liq: "75%", source: "Pyth" },
  { symbol: "SPYx", ltv: "70%", liq: "80%", source: "Pyth" },
  { symbol: "TSLAx", ltv: "50%", liq: "65%", source: "Pyth" },
];

const milestones = [
  ["02", "Markets + faucet", "Built"],
  ["03", "Collateral vault", "Built"],
  ["04", "Pyth valuation", "Built"],
  ["05", "Risk engine", "Built"],
];

export default function Home() {
  return (
    <main>
      <header>
        <div>
          <p className="eyebrow">SOLANA DEVNET · COLLATERALIZED CREDIT</p>
          <h1>44 Milady</h1>
          <p className="lede">
            Keep the exposure. Unlock the liquidity. Deposit market-linked collateral,
            value it through Pyth and measure borrowing power without selling it.
          </p>
        </div>
        <button disabled>Wallet wiring after program deploy</button>
      </header>

      <section className="grid">
        <article><span>Network</span><strong>Solana Devnet</strong></article>
        <article><span>Collateral slots</span><strong>8 / account</strong></article>
        <article><span>Valuation</span><strong>Pyth · on-chain checked</strong></article>
        <article><span>Borrowing</span><strong>Milestone 6</strong></article>
      </section>

      <section className="panel">
        <p className="eyebrow">MILESTONES 2—5</p>
        <h2>Collateral and risk core</h2>
        <div className="milestone-list">
          {milestones.map(([number, label, status]) => (
            <div className="milestone-row" key={number}>
              <span>{number}</span><strong>{label}</strong><em>{status}</em>
            </div>
          ))}
        </div>
      </section>

      <section className="panel">
        <p className="eyebrow">DEVNET MARKETS</p>
        <h2>Risk configuration preview</h2>
        <div className="market-table">
          {markets.map((market) => (
            <div className="market-row" key={market.symbol}>
              <strong>{market.symbol}</strong>
              <span>LTV {market.ltv}</span>
              <span>Liquidation {market.liq}</span>
              <span>{market.source}</span>
            </div>
          ))}
        </div>
      </section>
    </main>
  );
}
