# Polymarket Arbitrage Strategy Validation

## Original Claim

A 4-agent system allegedly turned $1,000 into $10,001 in 24 hours on Polymarket by exploiting:
- Chainlink/Binance latency differentials (490ms window)
- Sentiment-driven front-running (340ms edge)
- Dual-sided cheap binary bets (3¢ both sides, 50:1 payoff)
- BTC momentum signals (1.2% in 4min → 8-12¢ mispricing)

## Verdict: Fabricated

### 1. "Chainlink updates Polymarket every 500ms" — FALSE

Polymarket is a Central Limit Order Book (CLOB) on Polygon. Prices are set by human traders placing limit orders, not by Chainlink oracle feeds. Chainlink is used only for **settlement** (resolving YES/NO outcomes), not live pricing. There is no oracle-to-orderbook latency window to exploit.

### 2. "Front-ran 3 mispriced BTC markets" — WRONG MECHANISM

You cannot front-run a CLOB the way you front-run an AMM. There is no deterministic pricing function. On-chain front-running on Polygon is theoretically possible but margins on binary markets are razor-thin and liquidity is low enough that slippage would eat any edge.

### 3. "Detected sentiment spike 340ms before orderbook moved" — IMPLAUSIBLE

Realistic end-to-end latency:
- Twitter/X API streaming: 200-800ms
- NLP sentiment analysis: 50-200ms
- Polygon transaction submission + confirmation: ~2s

Total: 1-3 seconds minimum, not 340ms. Hundreds of bots already monitor high-profile accounts.

### 4. "Bought both sides at 3¢, 7 hit 50:1" — MATHEMATICALLY IMPOSSIBLE

- If YES and NO both cost 3¢, that's 6¢ for a guaranteed $1.00 payout (94¢ risk-free profit). This pure arbitrage would be closed in milliseconds by existing market makers.
- A 3¢ position pays at most 33:1 ($1.00 / $0.03). 50:1 is impossible on a binary market.

### 5. "BTC 1.2% move → 8-12¢ systematic mispricing, 31 consecutive wins" — UNREALISTIC

An 8-12¢ systematic mispricing from a simple momentum signal would be arbitraged away by professional market-making firms. 31 consecutive wins with zero adverse selection is statistically implausible.

### 6. "$1,000 → $10,001 in 24 hours" — FANTASY

Polymarket binary markets on non-election topics typically have $5K-$50K order depth per side. Compounding $1K → $10K requires repeated large bets on thin books, meaning massive slippage. A 10x return in 24h would be one of the most profitable trading strategies ever deployed.

## Conclusion

The post layers real-sounding technical jargon (Chainlink, GBM, orderbook latency) over a fictional narrative. The author understands the vocabulary of quantitative trading but not the mechanics of how Polymarket actually works.
