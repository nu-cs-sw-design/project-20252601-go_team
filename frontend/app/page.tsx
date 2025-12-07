"use client"

import OrderForm, { OrderFormValues } from "./order-form"
import { OrderbookDepthChart, DepthPoint } from "./orderbook-graph"

const ASSETS: string[] = [
  "BTC",
  "ETH",
  "SOL",
  "AAPL",
  "TSLA",
  "EUR/USD",
  "GOLD",
  "OIL",
  "SPY",
]

function mockDepth(symbol: string): { bids: DepthPoint[]; asks: DepthPoint[] } {
  const base = Math.random() * 100 + 50
  const bids = [
    { price: base - 2, volume: 5 },
    { price: base - 1, volume: 8 },
    { price: base - 0.5, volume: 3 },
  ]
  const asks = [
    { price: base + 0.5, volume: 4 },
    { price: base + 1, volume: 7 },
    { price: base + 2, volume: 2 },
  ]
  return { bids, asks }
}

export default function HomePage() {
  const handleSubmit = (values: OrderFormValues) => {
    console.log("Order form submitted:", values)
  }

  return (
    <div className="p-4 space-y-8">
      <h1 className="text-xl font-semibold">Order Entry & Depth</h1>
      <div className="grid gap-6 md:grid-cols-3">
        {ASSETS.map((symbol) => {
          const { bids, asks } = mockDepth(symbol)
          return (
            <div key={symbol} className="border rounded p-3 space-y-4">
              <h2 className="font-medium">{symbol}</h2>
              <OrderForm assets={[symbol]} maxQuantity={1_000_000} onSubmit={handleSubmit} />
              <OrderbookDepthChart bids={bids} asks={asks} className="w-full" />
            </div>
          )
        })}
      </div>
    </div>
  )
}