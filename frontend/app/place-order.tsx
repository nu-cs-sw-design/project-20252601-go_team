"use client"

import OrderForm, { OrderFormValues } from "./order-form"

export default function PlaceOrderPage() {
	const assets = ["AAPL", "MSFT", "GOOG"]

	const handleSubmit = (values: OrderFormValues) => {
		// For now, just log. Wire to backend later.
		console.log("Order form submitted:", values)
	}

	return (
		<div className="p-4">
			<h1 className="text-lg font-semibold mb-3">Place Order</h1>
			<OrderForm assets={assets} maxQuantity={1_000_000} onSubmit={handleSubmit} />
		</div>
	)
}
