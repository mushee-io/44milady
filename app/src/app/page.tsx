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
  ["06", "USDG liquidity + borrowing", "Built"],
  ["07", "Interest engine + reserves", "Built"],
];

const borrowFlow = [
  "Supply USDG",
  "Deposit collateral",
  "Read fresh Pyth prices",
  "Calculate LTV + health",
  "Borrow USDG",
];

export default function Home() {
  return (
    <main>
      <header>
        <div>
          <p className="eyebrow">SOLANA DEVNET · COLLATERALIZED CREDIT</p>
          <h1>44 Milady</h1>
          <p className="lede">
            Keep the exposure. Unlock the liquidity. Supply USDG or deposit market-linked
            collateral and borrow against it without selling the underlying exposure.
          </p>
        </div>
        <button disabled>Devnet deployment pending</button>
      </header>

      <section className="grid">
        <article><span>Network</span><strong>Solana Devnet</strong></article>
        <article><span>Liquidity</span><strong>USDG pool</strong></article>
        <article><span>Valuation</span><strong>Pyth · on-chain checked</strong></article>
        <article><span>Rates</span><strong>Dynamic · utilization based</strong></article>
      </section>

      <section className="panel">
        <p className="eyebrow">MILESTONES 2—7</p>
        <h2>Credit market core</h2>
        <div className="milestone-list">
          {milestones.map(([number, label, status]) => (
            <div className="milestone-row" key={number}>
              <span>{number}</span><strong>{label}</strong><em>{status}</em>
            </div>
          ))}
        </div>
      </section>

      <section className="panel">
        <p className="eyebrow">BORROW FLOW</p>
        <h2>Liquidity meets collateral.</h2>
        <div className="milestone-list">
          {borrowFlow.map((label, index) => (
            <div className="milestone-row" key={label}>
              <span>{String(index + 1).padStart(2, "0")}</span>
              <strong>{label}</strong>
              <em>{index === borrowFlow.length - 1 ? "USDG" : "→"}</em>
            </div>
          ))}
        </div>
      </section>

      <section className="panel">
        <p className="eyebrow">INTEREST MODEL</p>
        <h2>Rates move with utilization.</h2>
        <div className="market-table">
          <div className="market-row"><strong>Base</strong><span>2.00%</span><span>Kink 80%</span><span>Reserve 10%</span></div>
          <div className="market-row"><strong>At kink</strong><span>Borrow 10.00%</span><span>Supply 7.20%</span><span>Indexed</span></div>
          <div className="market-row"><strong>90% utilized</strong><span>Borrow 35.00%</span><span>Jump slope</span><span>Dynamic</span></div>
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
