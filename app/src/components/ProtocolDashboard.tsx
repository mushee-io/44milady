"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { Connection, PublicKey } from "@solana/web3.js";

type InjectedWallet = {
  publicKey?: PublicKey;
  isPhantom?: boolean;
  connect: () => Promise<{ publicKey: PublicKey }>;
  disconnect: () => Promise<void>;
};

declare global {
  interface Window {
    solana?: InjectedWallet;
    phantom?: { solana?: InjectedWallet };
  }
}

const markets = [
  { symbol: "NVDAx", ltv: 60, liquidation: 75, bonus: 5 },
  { symbol: "AAPLx", ltv: 60, liquidation: 75, bonus: 5 },
  { symbol: "SPYx", ltv: 70, liquidation: 80, bonus: 4 },
  { symbol: "TSLAx", ltv: 50, liquidation: 65, bonus: 7 },
];

function short(value: string) {
  return value.length > 14 ? value.slice(0, 6) + "…" + value.slice(-5) : value;
}

export default function ProtocolDashboard() {
  const rpcUrl = process.env.NEXT_PUBLIC_SOLANA_RPC_URL || "https://api.devnet.solana.com";
  const programId = process.env.NEXT_PUBLIC_PROGRAM_ID || "";
  const [wallet, setWallet] = useState("");
  const [programLive, setProgramLive] = useState<boolean | null>(null);
  const [active, setActive] = useState("Overview");
  const connection = useMemo(() => new Connection(rpcUrl, "confirmed"), [rpcUrl]);

  const refreshProgram = useCallback(async () => {
    if (!programId) {
      setProgramLive(false);
      return;
    }
    try {
      const info = await connection.getAccountInfo(new PublicKey(programId), "confirmed");
      setProgramLive(Boolean(info?.executable));
    } catch {
      setProgramLive(false);
    }
  }, [connection, programId]);

  useEffect(() => {
    refreshProgram();
    const timer = setInterval(refreshProgram, 15000);
    return () => clearInterval(timer);
  }, [refreshProgram]);

  async function connect() {
    const provider = window.phantom?.solana || window.solana;
    if (!provider) {
      window.open("https://phantom.app/", "_blank", "noopener,noreferrer");
      return;
    }
    const result = await provider.connect();
    setWallet(result.publicKey.toBase58());
  }

  async function disconnect() {
    const provider = window.phantom?.solana || window.solana;
    await provider?.disconnect();
    setWallet("");
  }

  const nav = ["Overview", "Markets", "Supply", "Borrow", "Portfolio", "Liquidations", "Faucet"];

  return (
    <main className="shell">
      <aside className="sidebar">
        <div className="brand">
          <span>44</span>
          <strong>MILADY</strong>
        </div>
        <nav>
          {nav.map((item) => (
            <button
              key={item}
              className={active === item ? "nav-active" : ""}
              onClick={() => setActive(item)}
            >
              {item}
            </button>
          ))}
        </nav>
        <div className="side-meta">
          <span>SOLANA DEVNET</span>
          <span>M1—M10</span>
        </div>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="kicker">COLLATERALIZED CREDIT / TOKENIZED MARKETS</p>
            <h1>{active}</h1>
          </div>
          <div className="top-actions">
            <span className={programLive ? "status live" : "status"}>
              {programLive ? "PROGRAM LIVE" : programId ? "PROGRAM OFFLINE" : "AWAITING DEPLOYMENT"}
            </span>
            <button className="wallet" onClick={wallet ? disconnect : connect}>
              {wallet ? short(wallet) : "CONNECT WALLET"}
            </button>
          </div>
        </header>

        <section className="hero-grid">
          <article className="hero-card dark">
            <span>44 MILADY</span>
            <h2>Keep the asset.<br />Unlock the liquidity.</h2>
            <p>Borrow USDG against tokenized market exposure without selling it.</p>
          </article>
          <article className="metric-card">
            <span>Borrow APR</span>
            <strong>Dynamic</strong>
            <small>2% base · 80% kink</small>
          </article>
          <article className="metric-card">
            <span>Liquidation</span>
            <strong>1.00 HF</strong>
            <small>50% default close factor</small>
          </article>
          <article className="metric-card">
            <span>Backstop</span>
            <strong>3 layers</strong>
            <small>Collateral · reserves · insurance</small>
          </article>
        </section>

        <section className="panel wide">
          <div className="panel-head">
            <div>
              <p className="kicker">MARKETS</p>
              <h3>Borrowing power</h3>
            </div>
            <span>PYTH-VALUED</span>
          </div>
          <div className="data-table">
            <div className="data-row data-head">
              <span>Market</span><span>Max LTV</span><span>Liquidation</span><span>Bonus</span><span>Status</span>
            </div>
            {markets.map((market) => (
              <div className="data-row" key={market.symbol}>
                <strong>{market.symbol}</strong>
                <span>{market.ltv}%</span>
                <span>{market.liquidation}%</span>
                <span>{market.bonus}%</span>
                <span className="pill">DEVNET READY</span>
              </div>
            ))}
          </div>
        </section>

        <section className="two-column">
          <article className="panel">
            <div className="panel-head">
              <div><p className="kicker">CREDIT ENGINE</p><h3>Position lifecycle</h3></div>
            </div>
            <ol className="flow">
              <li><span>01</span>Claim test assets</li>
              <li><span>02</span>Deposit collateral</li>
              <li><span>03</span>Pyth health valuation</li>
              <li><span>04</span>Borrow USDG</li>
              <li><span>05</span>Repay / withdraw / close</li>
            </ol>
          </article>

          <article className="panel action-panel">
            <p className="kicker">RELEASE CONTROL</p>
            <h3>{programLive ? "Devnet detected." : "Credential gate active."}</h3>
            <p>
              {programLive
                ? "The configured program account is executable on Solana Devnet."
                : "Write actions stay gated until the canonical program ID is deployed and public addresses are configured."}
            </p>
            <div className="address">
              <span>PROGRAM</span>
              <code>{programId ? short(programId) : "NOT SET"}</code>
            </div>
            <button className="primary" disabled={!programLive || !wallet}>
              {wallet ? "OPEN PROTOCOL ACTIONS" : "CONNECT WALLET FIRST"}
            </button>
          </article>
        </section>

        <footer>
          <span>44 MILADY / DEVNET RELEASE CANDIDATE</span>
          <span>PYTH · SOLANA · USDG</span>
        </footer>
      </section>
    </main>
  );
}
