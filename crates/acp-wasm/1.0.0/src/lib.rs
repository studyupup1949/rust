//! WebAssembly bindings for StateSet ACP Handler
//!
//! This crate provides browser-compatible WASM bindings for the ACP checkout service.
//!
//! # Example (JavaScript)
//!
//! ```javascript
//! import init, { AcpClient } from '@stateset/acp-wasm';
//!
//! await init();
//!
//! const client = new AcpClient('api_key_demo_123');
//!
//! const session = await client.createCheckoutSession([
//!     { id: 'prod_laptop_001', quantity: 1 }
//! ]);
//!
//! console.log('Session ID:', session.id);
//!
//! const result = await client.completeCheckoutSession(session.id, 'tok_demo');
//! console.log('Order ID:', result.order.id);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

// Global session store
static SESSIONS: RwLock<Option<HashMap<String, CheckoutSessionInternal>>> = RwLock::new(None);

fn get_sessions() -> &'static RwLock<Option<HashMap<String, CheckoutSessionInternal>>> {
    &SESSIONS
}

fn with_sessions<F, R>(f: F) -> Result<R, JsValue>
where
    F: FnOnce(&mut HashMap<String, CheckoutSessionInternal>) -> Result<R, String>,
{
    let mut guard = get_sessions()
        .write()
        .map_err(|_| JsValue::from_str("Lock error"))?;

    if guard.is_none() {
        *guard = Some(HashMap::new());
    }

    let sessions = guard.as_mut().unwrap();
    f(sessions).map_err(|e| JsValue::from_str(&e))
}

// Internal types
#[derive(Clone, Serialize, Deserialize)]
struct MoneyInternal {
    amount: i64,
    currency: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct LineItemInternal {
    id: String,
    title: String,
    quantity: i32,
    unit_price: MoneyInternal,
    variant_id: Option<String>,
    sku: Option<String>,
    image_url: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct TotalsInternal {
    subtotal: MoneyInternal,
    tax: MoneyInternal,
    shipping: MoneyInternal,
    discount: MoneyInternal,
    grand_total: MoneyInternal,
}

#[derive(Clone, Serialize, Deserialize)]
struct CustomerInternal {
    billing_address: Option<AddressInternal>,
    shipping_address: Option<AddressInternal>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AddressInternal {
    name: Option<String>,
    line1: Option<String>,
    line2: Option<String>,
    city: Option<String>,
    region: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    phone: Option<String>,
    email: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct CheckoutSessionInternal {
    id: String,
    status: String,
    items: Vec<LineItemInternal>,
    totals: TotalsInternal,
    customer: Option<CustomerInternal>,
    created_at: String,
    updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct OrderInternal {
    id: String,
    checkout_session_id: String,
    status: String,
    permalink_url: Option<String>,
}

// Product catalog
fn get_product(id: &str) -> Option<LineItemInternal> {
    match id {
        "prod_laptop_001" => Some(LineItemInternal {
            id: id.to_string(),
            title: "MacBook Pro 14\"".to_string(),
            quantity: 1,
            unit_price: MoneyInternal {
                amount: 199900,
                currency: "USD".to_string(),
            },
            variant_id: Some("var_001".to_string()),
            sku: Some("MBP14-M3".to_string()),
            image_url: Some("https://example.com/mbp14.jpg".to_string()),
        }),
        "prod_mouse_002" => Some(LineItemInternal {
            id: id.to_string(),
            title: "Magic Mouse".to_string(),
            quantity: 1,
            unit_price: MoneyInternal {
                amount: 9900,
                currency: "USD".to_string(),
            },
            variant_id: Some("var_002".to_string()),
            sku: Some("MM-WHITE".to_string()),
            image_url: Some("https://example.com/mouse.jpg".to_string()),
        }),
        "prod_keyboard_003" => Some(LineItemInternal {
            id: id.to_string(),
            title: "Magic Keyboard".to_string(),
            quantity: 1,
            unit_price: MoneyInternal {
                amount: 29900,
                currency: "USD".to_string(),
            },
            variant_id: Some("var_003".to_string()),
            sku: Some("MK-SILVER".to_string()),
            image_url: Some("https://example.com/keyboard.jpg".to_string()),
        }),
        _ => None,
    }
}

fn calculate_totals(items: &[LineItemInternal]) -> TotalsInternal {
    let subtotal: i64 = items
        .iter()
        .map(|i| i.unit_price.amount * i.quantity as i64)
        .sum();
    let tax = (subtotal as f64 * 0.0875) as i64;
    let shipping = if subtotal > 10000 { 0 } else { 999 };
    let grand_total = subtotal + tax + shipping;

    TotalsInternal {
        subtotal: MoneyInternal {
            amount: subtotal,
            currency: "USD".to_string(),
        },
        tax: MoneyInternal {
            amount: tax,
            currency: "USD".to_string(),
        },
        shipping: MoneyInternal {
            amount: shipping,
            currency: "USD".to_string(),
        },
        discount: MoneyInternal {
            amount: 0,
            currency: "USD".to_string(),
        },
        grand_total: MoneyInternal {
            amount: grand_total,
            currency: "USD".to_string(),
        },
    }
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// ACP Client for browser environments.
#[wasm_bindgen]
pub struct AcpClient {
    api_key: Option<String>,
}

#[wasm_bindgen]
impl AcpClient {
    /// Creates a new ACP client.
    #[wasm_bindgen(constructor)]
    pub fn new(api_key: Option<String>) -> AcpClient {
        AcpClient { api_key }
    }

    /// Creates a new checkout session.
    ///
    /// @param items - Array of items with id and quantity
    /// @returns Promise resolving to the created checkout session
    #[wasm_bindgen(js_name = createCheckoutSession)]
    pub fn create_checkout_session(&self, items: JsValue) -> Result<JsValue, JsValue> {
        #[derive(Deserialize)]
        struct RequestItem {
            id: String,
            quantity: i32,
        }

        let items: Vec<RequestItem> = serde_wasm_bindgen::from_value(items)?;

        if items.is_empty() {
            return Err(JsValue::from_str("At least one item is required"));
        }

        let mut line_items = Vec::new();
        for item in items {
            if let Some(mut product) = get_product(&item.id) {
                product.quantity = item.quantity;
                line_items.push(product);
            } else {
                return Err(JsValue::from_str(&format!(
                    "Product not found: {}",
                    item.id
                )));
            }
        }

        let now = now_iso();
        let totals = calculate_totals(&line_items);

        let session = CheckoutSessionInternal {
            id: Uuid::new_v4().to_string(),
            status: "not_ready_for_payment".to_string(),
            items: line_items,
            totals,
            customer: None,
            created_at: now.clone(),
            updated_at: now,
        };

        with_sessions(|sessions| {
            sessions.insert(session.id.clone(), session.clone());
            Ok(())
        })?;

        serde_wasm_bindgen::to_value(&session).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Gets an existing checkout session.
    #[wasm_bindgen(js_name = getCheckoutSession)]
    pub fn get_checkout_session(&self, session_id: &str) -> Result<JsValue, JsValue> {
        with_sessions(|sessions| {
            sessions
                .get(session_id)
                .cloned()
                .ok_or_else(|| format!("Session not found: {}", session_id))
        })
        .and_then(|session| {
            serde_wasm_bindgen::to_value(&session).map_err(|e| JsValue::from_str(&e.to_string()))
        })
    }

    /// Updates a checkout session.
    #[wasm_bindgen(js_name = updateCheckoutSession)]
    pub fn update_checkout_session(
        &self,
        session_id: &str,
        items: Option<JsValue>,
        customer: Option<JsValue>,
    ) -> Result<JsValue, JsValue> {
        #[derive(Deserialize)]
        struct RequestItem {
            id: String,
            quantity: i32,
        }

        with_sessions(|sessions| {
            let session = sessions
                .get_mut(session_id)
                .ok_or_else(|| format!("Session not found: {}", session_id))?;

            if session.status == "completed" || session.status == "canceled" {
                return Err("Cannot update completed or canceled session".to_string());
            }

            // Update items if provided
            if let Some(items_val) = items {
                if !items_val.is_undefined() && !items_val.is_null() {
                    let items: Vec<RequestItem> = serde_wasm_bindgen::from_value(items_val)
                        .map_err(|e| e.to_string())?;

                    let mut line_items = Vec::new();
                    for item in items {
                        if let Some(mut product) = get_product(&item.id) {
                            product.quantity = item.quantity;
                            line_items.push(product);
                        } else {
                            return Err(format!("Product not found: {}", item.id));
                        }
                    }
                    session.items = line_items;
                    session.totals = calculate_totals(&session.items);
                }
            }

            // Update customer if provided
            if let Some(customer_val) = customer {
                if !customer_val.is_undefined() && !customer_val.is_null() {
                    session.customer = serde_wasm_bindgen::from_value(customer_val)
                        .map_err(|e| e.to_string())?;
                }
            }

            session.status = "ready_for_payment".to_string();
            session.updated_at = now_iso();

            Ok(session.clone())
        })
        .and_then(|session| {
            serde_wasm_bindgen::to_value(&session).map_err(|e| JsValue::from_str(&e.to_string()))
        })
    }

    /// Completes a checkout session with payment.
    #[wasm_bindgen(js_name = completeCheckoutSession)]
    pub fn complete_checkout_session(
        &self,
        session_id: &str,
        payment_token: &str,
    ) -> Result<JsValue, JsValue> {
        #[derive(Serialize)]
        struct CompletionResult {
            session: CheckoutSessionInternal,
            order: OrderInternal,
        }

        with_sessions(|sessions| {
            let session = sessions
                .get_mut(session_id)
                .ok_or_else(|| format!("Session not found: {}", session_id))?;

            if session.status == "completed" {
                return Err("Session already completed".to_string());
            }

            if session.status == "canceled" {
                return Err("Cannot complete canceled session".to_string());
            }

            session.status = "completed".to_string();
            session.updated_at = now_iso();

            let order = OrderInternal {
                id: Uuid::new_v4().to_string(),
                checkout_session_id: session.id.clone(),
                status: "placed".to_string(),
                permalink_url: Some(format!("https://orders.example.com/{}", session.id)),
            };

            Ok(CompletionResult {
                session: session.clone(),
                order,
            })
        })
        .and_then(|result| {
            serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
        })
    }

    /// Cancels a checkout session.
    #[wasm_bindgen(js_name = cancelCheckoutSession)]
    pub fn cancel_checkout_session(&self, session_id: &str) -> Result<JsValue, JsValue> {
        with_sessions(|sessions| {
            let session = sessions
                .get_mut(session_id)
                .ok_or_else(|| format!("Session not found: {}", session_id))?;

            if session.status == "completed" {
                return Err("Cannot cancel completed session".to_string());
            }

            session.status = "canceled".to_string();
            session.updated_at = now_iso();

            Ok(session.clone())
        })
        .and_then(|session| {
            serde_wasm_bindgen::to_value(&session).map_err(|e| JsValue::from_str(&e.to_string()))
        })
    }
}

/// Get library version.
#[wasm_bindgen]
pub fn version() -> String {
    "1.0.0".to_string()
}

/// Initialize the WASM module (call once at startup).
#[wasm_bindgen(start)]
pub fn init() {
    // Initialize panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
