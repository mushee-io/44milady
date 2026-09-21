import type { Metadata } from "next";
import "./styles.css";

export const metadata: Metadata = {
  title: "44 Milady",
  description: "Collateralized markets on Solana",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
