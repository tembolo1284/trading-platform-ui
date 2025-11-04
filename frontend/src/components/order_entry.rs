// frontend/src/components/order_entry.rs
use leptos::*;
use crate::api::TradingClient;

#[component]
pub fn OrderEntry() -> impl IntoView {
    let (symbol, set_symbol) = create_signal("AAPL".to_string());
    let (price, set_price) = create_signal(150.0);
    let (quantity, set_quantity) = create_signal(100);
    let (side, set_side) = create_signal(Side::Buy);
    
    let submit_order = create_action(|order: &OrderRequest| {
        let client = use_context::<TradingClient>().unwrap();
        async move {
            client.submit_order(order.clone()).await
        }
    });
    
    view! {
        <div class="order-entry">
            <h2>"Place Order"</h2>
            
            <input 
                type="text"
                placeholder="Symbol"
                on:input=move |ev| set_symbol(event_target_value(&ev))
                prop:value=symbol
            />
            
            <input 
                type="number"
                placeholder="Price"
                on:input=move |ev| set_price(event_target_value(&ev).parse().unwrap_or(0.0))
                prop:value=price
            />
            
            <input 
                type="number"
                placeholder="Quantity"
                on:input=move |ev| set_quantity(event_target_value(&ev).parse().unwrap_or(0))
                prop:value=quantity
            />
            
            <select on:change=move |ev| {
                set_side(if event_target_value(&ev) == "BUY" { Side::Buy } else { Side::Sell })
            }>
                <option value="BUY">"Buy"</option>
                <option value="SELL">"Sell"</option>
            </select>
            
            <button on:click=move |_| {
                submit_order.dispatch(OrderRequest {
                    symbol: symbol.get(),
                    price: price.get(),
                    quantity: quantity.get(),
                    side: side.get() as i32,
                    ..Default::default()
                });
            }>
                "Submit Order"
            </button>
        </div>
    }
}
