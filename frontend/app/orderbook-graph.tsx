"use client"

import { useMemo } from "react"
import { Bar, BarChart, CartesianGrid, Tooltip, XAxis, YAxis } from "recharts"

// A minimal, reusable depth chart for order books.
// Price on X-axis, volume on Y-axis. Two series: bids and asks.

export type DepthPoint = {
  price: number
  volume: number
}

export type OrderbookDepthProps = {
  bids: DepthPoint[]
  asks: DepthPoint[]
  className?: string
}

// Normalize and sort input for a clean chart. Keep it simple.
function useDepthData(bids: DepthPoint[], asks: DepthPoint[]) {
  return useMemo(() => {
    const round3 = (x: number) => Number(x.toFixed(3))

    const sortedBids = [...bids]
      .filter((b) => Number.isFinite(b.price) && Number.isFinite(b.volume))
      .map((b) => ({ price: round3(b.price), volume: round3(b.volume) }))
      .sort((a, b) => a.price - b.price)

    const sortedAsks = [...asks]
      .filter((a) => Number.isFinite(a.price) && Number.isFinite(a.volume))
      .map((a) => ({ price: round3(a.price), volume: round3(a.volume) }))
      .sort((a, b) => a.price - b.price)

    const byPrice = new Map<number, { price: number; bid?: number; ask?: number }>()

    for (const b of sortedBids) {
      const existing = byPrice.get(b.price) || { price: b.price }
      existing.bid = round3((existing.bid || 0) + b.volume)
      byPrice.set(b.price, existing)
    }

    for (const a of sortedAsks) {
      const existing = byPrice.get(a.price) || { price: a.price }
      existing.ask = round3((existing.ask || 0) + a.volume)
      byPrice.set(a.price, existing)
    }

    const data = Array.from(byPrice.values()).sort((a, b) => a.price - b.price)
    return data
  }, [bids, asks])
}


export function OrderbookDepthChart({ bids, asks, className }: OrderbookDepthProps) {
  const data = useDepthData(bids, asks)

  return (
    <div className={className || "min-h-[200px] w-full"}>
      <BarChart width={600} height={300} data={data}>
        <CartesianGrid vertical={false} strokeDasharray="3 3" />
        <XAxis
          dataKey="price"
          tickLine={false}
          axisLine={false}
          label={{ value: "Price", position: "insideBottom", offset: -5 }}
        />
        <YAxis
          tickLine={false}
          axisLine={false}
          label={{ value: "Volume", angle: -90, position: "insideLeft" }}
        />
        <Tooltip formatter={(value: number, name) => [value, name]} />
        <Bar dataKey="bid" fill="#22c55e" />
        <Bar dataKey="ask" fill="#ef4444" />
      </BarChart>
    </div>
  )
}

// Backwards-compatible export name used elsewhere.
export const OrderbookChart = OrderbookDepthChart
