"use client"

import { useState } from "react"

type OrderSide = "BUY" | "SELL"
type OrderType = "LIMIT" | "MARKET"
type TimeInForce = "GTC" | "IOC" | "FOK"

export type OrderFormValues = {
	symbol: string
	side: OrderSide
	orderType: OrderType
	quantity: number
	tif: TimeInForce
	price?: number
}

export type OrderFormProps = {
	assets: string[]
	maxQuantity?: number
	onSubmit: (values: OrderFormValues) => void
	className?: string
}

export function OrderForm({ assets, maxQuantity = 1_000_000, onSubmit, className }: OrderFormProps) {
	const [symbol, setSymbol] = useState(assets[0] ?? "")
	const [side, setSide] = useState<OrderSide>("BUY")
	const [orderType, setOrderType] = useState<OrderType>("LIMIT")
	const [quantity, setQuantity] = useState<number>(0)
	const [tif, setTif] = useState<TimeInForce>("GTC")
	const [price, setPrice] = useState<number | undefined>(undefined)

	const handleSubmit = (e?: React.FormEvent) => {
		e?.preventDefault()
		const payload: OrderFormValues = {
			symbol,
			side,
			orderType,
			quantity,
			tif,
			...(orderType === "LIMIT" && price ? { price } : {}),
		}
		// Local debug log to verify submission firing
		console.log("OrderForm submit payload:", payload)
		onSubmit(payload)
	}

	const isLimit = orderType === "LIMIT"

	return (
		<form onSubmit={(e) => handleSubmit(e)} className={className ?? "space-y-3 max-w-sm"}>
			{/* Asset */}
			<label className="block">
				<span className="text-sm">Asset</span>
				<select
					className="mt-1 w-full border rounded px-2 py-1"
					value={symbol}
					onChange={(e) => setSymbol(e.target.value)}
				>
					{assets.map((a) => (
						<option key={a} value={a}>
							{a}
						</option>
					))}
				</select>
			</label>

			{/* Side */}
			<label className="block">
				<span className="text-sm">Side</span>
				<select
					className="mt-1 w-full border rounded px-2 py-1"
					value={side}
					onChange={(e) => setSide(e.target.value as OrderSide)}
				>
					<option value="BUY">BUY</option>
					<option value="SELL">SELL</option>
				</select>
			</label>

			{/* Order Type */}
			<label className="block">
				<span className="text-sm">Order Type</span>
				<select
					className="mt-1 w-full border rounded px-2 py-1"
					value={orderType}
					onChange={(e) => setOrderType(e.target.value as OrderType)}
				>
					<option value="LIMIT">Limit Order</option>
					<option value="MARKET">Market Order</option>
				</select>
			</label>

			{/* Quantity */}
			<label className="block">
				<span className="text-sm">Volume</span>
				<input
					type="number"
					min={0}
					max={maxQuantity}
					step={0.0001}
					className="mt-1 w-full border rounded px-2 py-1"
					value={Number.isFinite(quantity) ? quantity : 0}
					onChange={(e) => setQuantity(Number(e.target.value))}
				/>
			</label>

			{/* TIF */}
			<label className="block">
				<span className="text-sm">Time In Force</span>
				<select
					className="mt-1 w-full border rounded px-2 py-1"
					value={tif}
					onChange={(e) => setTif(e.target.value as TimeInForce)}
				>
					{/* For Market, GTC is allowed but coerced to IOC server-side; keep simple */}
					<option value="GTC">GTC</option>
					<option value="IOC">IOC</option>
					<option value="FOK">FOK</option>
				</select>
			</label>

			{/* Price (only for Limit) */}
			{isLimit && (
				<label className="block">
					<span className="text-sm">Price</span>
					<input
						type="number"
						min={0}
						step={0.0001}
						className="mt-1 w-full border rounded px-2 py-1"
						value={price ?? 0}
						onChange={(e) => setPrice(Number(e.target.value))}
					/>
				</label>
			)}

			<button
				type="submit"
				onClick={() => handleSubmit()}
				className="border rounded px-3 py-1 cursor-pointer bg-white hover:bg-gray-100 active:bg-gray-200 transition-colors shadow-sm hover:shadow active:scale-[0.98]"
			>
				Place Order
			</button>
		</form>
	)
}

export default OrderForm
