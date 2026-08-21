//! C FFI bindings for StateSet ACP Handler
//!
//! This crate provides C-compatible bindings for the ACP checkout service,
//! enabling integration with Node.js, Python, Go, Java, Ruby, PHP, Swift, .NET, and more.

use libc::{c_char, c_int, c_longlong, size_t};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::{Arc, Mutex, RwLock};
use tokio::runtime::Runtime;
use uuid::Uuid;

// Global Tokio runtime for async operations
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

// In-memory session store
static SESSIONS: Lazy<RwLock<HashMap<String, CheckoutSessionInternal>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

// ============================================================================
// Error Handling
// ============================================================================

/// Error codes returned by FFI functions
#[repr(C)]
pub enum AcpErrorCode {
    Success = 0,
    InvalidInput = 1,
    NotFound = 2,
    InvalidOperation = 3,
    InsufficientStock = 4,
    PaymentFailed = 5,
    InternalError = 6,
    NullPointer = 7,
    Utf8Error = 8,
    JsonError = 9,
}

/// Thread-local last error message
thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

fn set_last_error(msg: String) {
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
}

/// Get the last error message. Caller must free the returned string with acp_free_string.
#[no_mangle]
pub extern "C" fn acp_get_last_error() -> *mut c_char {
    LAST_ERROR.with(|e| {
        match e.borrow().as_ref() {
            Some(msg) => CString::new(msg.as_str()).unwrap().into_raw(),
            None => ptr::null_mut(),
        }
    })
}

/// Free a string allocated by this library
#[no_mangle]
pub extern "C" fn acp_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// ============================================================================
// Data Types
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckoutSessionStatus {
    NotReadyForPayment = 1,
    ReadyForPayment = 2,
    Completed = 3,
    Canceled = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Placed = 1,
    Failed = 2,
    Refunded = 3,
}

#[repr(C)]
pub struct AcpMoney {
    pub amount: c_longlong,
    pub currency: *mut c_char,
}

#[repr(C)]
pub struct AcpAddress {
    pub name: *mut c_char,
    pub line1: *mut c_char,
    pub line2: *mut c_char,
    pub city: *mut c_char,
    pub region: *mut c_char,
    pub postal_code: *mut c_char,
    pub country: *mut c_char,
    pub phone: *mut c_char,
    pub email: *mut c_char,
}

#[repr(C)]
pub struct AcpLineItem {
    pub id: *mut c_char,
    pub title: *mut c_char,
    pub quantity: c_int,
    pub unit_price_amount: c_longlong,
    pub unit_price_currency: *mut c_char,
    pub variant_id: *mut c_char,
    pub sku: *mut c_char,
    pub image_url: *mut c_char,
}

#[repr(C)]
pub struct AcpTotals {
    pub subtotal_amount: c_longlong,
    pub subtotal_currency: *mut c_char,
    pub tax_amount: c_longlong,
    pub tax_currency: *mut c_char,
    pub shipping_amount: c_longlong,
    pub shipping_currency: *mut c_char,
    pub discount_amount: c_longlong,
    pub discount_currency: *mut c_char,
    pub grand_total_amount: c_longlong,
    pub grand_total_currency: *mut c_char,
}

#[repr(C)]
pub struct AcpCheckoutSession {
    pub id: *mut c_char,
    pub status: CheckoutSessionStatus,
    pub items: *mut AcpLineItem,
    pub items_count: size_t,
    pub totals: AcpTotals,
    pub created_at: *mut c_char,
    pub updated_at: *mut c_char,
}

#[repr(C)]
pub struct AcpOrder {
    pub id: *mut c_char,
    pub checkout_session_id: *mut c_char,
    pub status: OrderStatus,
    pub permalink_url: *mut c_char,
}

#[repr(C)]
pub struct AcpCheckoutSessionWithOrder {
    pub session: AcpCheckoutSession,
    pub order: AcpOrder,
}

#[repr(C)]
pub struct AcpRequestItem {
    pub id: *const c_char,
    pub quantity: c_int,
}

// ============================================================================
// Internal Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MoneyInternal {
    amount: i64,
    currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LineItemInternal {
    id: String,
    title: String,
    quantity: i32,
    unit_price: MoneyInternal,
    variant_id: Option<String>,
    sku: Option<String>,
    image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TotalsInternal {
    subtotal: MoneyInternal,
    tax: MoneyInternal,
    shipping: MoneyInternal,
    discount: MoneyInternal,
    grand_total: MoneyInternal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckoutSessionInternal {
    id: String,
    status: String,
    items: Vec<LineItemInternal>,
    totals: TotalsInternal,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderInternal {
    id: String,
    checkout_session_id: String,
    status: String,
    permalink_url: Option<String>,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn str_to_cstring(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

fn cstr_to_string(s: *const c_char) -> Result<String, AcpErrorCode> {
    if s.is_null() {
        return Err(AcpErrorCode::NullPointer);
    }
    unsafe {
        CStr::from_ptr(s)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| AcpErrorCode::Utf8Error)
    }
}

fn status_from_str(s: &str) -> CheckoutSessionStatus {
    match s {
        "ready_for_payment" => CheckoutSessionStatus::ReadyForPayment,
        "completed" => CheckoutSessionStatus::Completed,
        "canceled" => CheckoutSessionStatus::Canceled,
        _ => CheckoutSessionStatus::NotReadyForPayment,
    }
}

fn order_status_from_str(s: &str) -> OrderStatus {
    match s {
        "placed" => OrderStatus::Placed,
        "refunded" => OrderStatus::Refunded,
        _ => OrderStatus::Failed,
    }
}

// Product catalog for demo
fn get_product(id: &str) -> Option<LineItemInternal> {
    match id {
        "prod_laptop_001" => Some(LineItemInternal {
            id: id.to_string(),
            title: "MacBook Pro 14\"".to_string(),
            quantity: 1,
            unit_price: MoneyInternal { amount: 199900, currency: "USD".to_string() },
            variant_id: Some("var_001".to_string()),
            sku: Some("MBP14-M3".to_string()),
            image_url: Some("https://example.com/mbp14.jpg".to_string()),
        }),
        "prod_mouse_002" => Some(LineItemInternal {
            id: id.to_string(),
            title: "Magic Mouse".to_string(),
            quantity: 1,
            unit_price: MoneyInternal { amount: 9900, currency: "USD".to_string() },
            variant_id: Some("var_002".to_string()),
            sku: Some("MM-WHITE".to_string()),
            image_url: Some("https://example.com/mouse.jpg".to_string()),
        }),
        "prod_keyboard_003" => Some(LineItemInternal {
            id: id.to_string(),
            title: "Magic Keyboard".to_string(),
            quantity: 1,
            unit_price: MoneyInternal { amount: 29900, currency: "USD".to_string() },
            variant_id: Some("var_003".to_string()),
            sku: Some("MK-SILVER".to_string()),
            image_url: Some("https://example.com/keyboard.jpg".to_string()),
        }),
        _ => None,
    }
}

fn calculate_totals(items: &[LineItemInternal]) -> TotalsInternal {
    let subtotal: i64 = items.iter().map(|i| i.unit_price.amount * i.quantity as i64).sum();
    let tax = (subtotal as f64 * 0.0875) as i64;
    let shipping = if subtotal > 10000 { 0 } else { 999 };
    let discount = 0;
    let grand_total = subtotal + tax + shipping - discount;

    TotalsInternal {
        subtotal: MoneyInternal { amount: subtotal, currency: "USD".to_string() },
        tax: MoneyInternal { amount: tax, currency: "USD".to_string() },
        shipping: MoneyInternal { amount: shipping, currency: "USD".to_string() },
        discount: MoneyInternal { amount: discount, currency: "USD".to_string() },
        grand_total: MoneyInternal { amount: grand_total, currency: "USD".to_string() },
    }
}

fn session_to_ffi(session: &CheckoutSessionInternal) -> AcpCheckoutSession {
    let items: Vec<AcpLineItem> = session.items.iter().map(|item| {
        AcpLineItem {
            id: str_to_cstring(&item.id),
            title: str_to_cstring(&item.title),
            quantity: item.quantity,
            unit_price_amount: item.unit_price.amount,
            unit_price_currency: str_to_cstring(&item.unit_price.currency),
            variant_id: item.variant_id.as_ref().map(|s| str_to_cstring(s)).unwrap_or(ptr::null_mut()),
            sku: item.sku.as_ref().map(|s| str_to_cstring(s)).unwrap_or(ptr::null_mut()),
            image_url: item.image_url.as_ref().map(|s| str_to_cstring(s)).unwrap_or(ptr::null_mut()),
        }
    }).collect();

    let items_ptr = if items.is_empty() {
        ptr::null_mut()
    } else {
        let boxed = items.into_boxed_slice();
        let ptr = boxed.as_ptr() as *mut AcpLineItem;
        std::mem::forget(boxed);
        ptr
    };

    AcpCheckoutSession {
        id: str_to_cstring(&session.id),
        status: status_from_str(&session.status),
        items: items_ptr,
        items_count: session.items.len(),
        totals: AcpTotals {
            subtotal_amount: session.totals.subtotal.amount,
            subtotal_currency: str_to_cstring(&session.totals.subtotal.currency),
            tax_amount: session.totals.tax.amount,
            tax_currency: str_to_cstring(&session.totals.tax.currency),
            shipping_amount: session.totals.shipping.amount,
            shipping_currency: str_to_cstring(&session.totals.shipping.currency),
            discount_amount: session.totals.discount.amount,
            discount_currency: str_to_cstring(&session.totals.discount.currency),
            grand_total_amount: session.totals.grand_total.amount,
            grand_total_currency: str_to_cstring(&session.totals.grand_total.currency),
        },
        created_at: str_to_cstring(&session.created_at),
        updated_at: str_to_cstring(&session.updated_at),
    }
}

// ============================================================================
// FFI Functions
// ============================================================================

/// Initialize the ACP library. Call once at startup.
#[no_mangle]
pub extern "C" fn acp_init() -> c_int {
    // Force runtime initialization
    let _ = &*RUNTIME;
    AcpErrorCode::Success as c_int
}

/// Shutdown the ACP library. Call before exit.
#[no_mangle]
pub extern "C" fn acp_shutdown() {
    // Clear sessions
    if let Ok(mut sessions) = SESSIONS.write() {
        sessions.clear();
    }
}

/// Create a new checkout session.
///
/// # Arguments
/// * `items` - Array of request items
/// * `items_count` - Number of items
/// * `out_session` - Output pointer for the created session
///
/// # Returns
/// Error code (0 = success)
#[no_mangle]
pub extern "C" fn acp_create_checkout_session(
    items: *const AcpRequestItem,
    items_count: size_t,
    out_session: *mut AcpCheckoutSession,
) -> c_int {
    if items.is_null() || out_session.is_null() {
        set_last_error("Null pointer provided".to_string());
        return AcpErrorCode::NullPointer as c_int;
    }

    if items_count == 0 {
        set_last_error("At least one item is required".to_string());
        return AcpErrorCode::InvalidInput as c_int;
    }

    let items_slice = unsafe { std::slice::from_raw_parts(items, items_count) };

    let mut line_items = Vec::new();
    for item in items_slice {
        let id = match cstr_to_string(item.id) {
            Ok(s) => s,
            Err(e) => {
                set_last_error("Invalid item ID".to_string());
                return e as c_int;
            }
        };

        if let Some(mut product) = get_product(&id) {
            product.quantity = item.quantity;
            line_items.push(product);
        } else {
            set_last_error(format!("Product not found: {}", id));
            return AcpErrorCode::NotFound as c_int;
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    let totals = calculate_totals(&line_items);

    let session = CheckoutSessionInternal {
        id: Uuid::new_v4().to_string(),
        status: "not_ready_for_payment".to_string(),
        items: line_items,
        totals,
        created_at: now.clone(),
        updated_at: now,
    };

    // Store session
    if let Ok(mut sessions) = SESSIONS.write() {
        sessions.insert(session.id.clone(), session.clone());
    }

    let ffi_session = session_to_ffi(&session);
    unsafe {
        *out_session = ffi_session;
    }

    AcpErrorCode::Success as c_int
}

/// Get a checkout session by ID.
#[no_mangle]
pub extern "C" fn acp_get_checkout_session(
    session_id: *const c_char,
    out_session: *mut AcpCheckoutSession,
) -> c_int {
    if session_id.is_null() || out_session.is_null() {
        set_last_error("Null pointer provided".to_string());
        return AcpErrorCode::NullPointer as c_int;
    }

    let id = match cstr_to_string(session_id) {
        Ok(s) => s,
        Err(e) => {
            set_last_error("Invalid session ID".to_string());
            return e as c_int;
        }
    };

    let sessions = match SESSIONS.read() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Failed to read sessions".to_string());
            return AcpErrorCode::InternalError as c_int;
        }
    };

    match sessions.get(&id) {
        Some(session) => {
            let ffi_session = session_to_ffi(session);
            unsafe {
                *out_session = ffi_session;
            }
            AcpErrorCode::Success as c_int
        }
        None => {
            set_last_error(format!("Session not found: {}", id));
            AcpErrorCode::NotFound as c_int
        }
    }
}

/// Update a checkout session.
#[no_mangle]
pub extern "C" fn acp_update_checkout_session(
    session_id: *const c_char,
    items: *const AcpRequestItem,
    items_count: size_t,
    out_session: *mut AcpCheckoutSession,
) -> c_int {
    if session_id.is_null() || out_session.is_null() {
        set_last_error("Null pointer provided".to_string());
        return AcpErrorCode::NullPointer as c_int;
    }

    let id = match cstr_to_string(session_id) {
        Ok(s) => s,
        Err(e) => {
            set_last_error("Invalid session ID".to_string());
            return e as c_int;
        }
    };

    let mut sessions = match SESSIONS.write() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Failed to write sessions".to_string());
            return AcpErrorCode::InternalError as c_int;
        }
    };

    let session = match sessions.get_mut(&id) {
        Some(s) => s,
        None => {
            set_last_error(format!("Session not found: {}", id));
            return AcpErrorCode::NotFound as c_int;
        }
    };

    if session.status == "completed" || session.status == "canceled" {
        set_last_error("Cannot update completed or canceled session".to_string());
        return AcpErrorCode::InvalidOperation as c_int;
    }

    // Update items if provided
    if !items.is_null() && items_count > 0 {
        let items_slice = unsafe { std::slice::from_raw_parts(items, items_count) };
        let mut line_items = Vec::new();

        for item in items_slice {
            let item_id = match cstr_to_string(item.id) {
                Ok(s) => s,
                Err(e) => return e as c_int,
            };

            if let Some(mut product) = get_product(&item_id) {
                product.quantity = item.quantity;
                line_items.push(product);
            } else {
                set_last_error(format!("Product not found: {}", item_id));
                return AcpErrorCode::NotFound as c_int;
            }
        }

        session.items = line_items;
        session.totals = calculate_totals(&session.items);
    }

    session.status = "ready_for_payment".to_string();
    session.updated_at = chrono::Utc::now().to_rfc3339();

    let ffi_session = session_to_ffi(session);
    unsafe {
        *out_session = ffi_session;
    }

    AcpErrorCode::Success as c_int
}

/// Complete a checkout session with payment.
#[no_mangle]
pub extern "C" fn acp_complete_checkout_session(
    session_id: *const c_char,
    payment_token: *const c_char,
    out_result: *mut AcpCheckoutSessionWithOrder,
) -> c_int {
    if session_id.is_null() || out_result.is_null() {
        set_last_error("Null pointer provided".to_string());
        return AcpErrorCode::NullPointer as c_int;
    }

    let id = match cstr_to_string(session_id) {
        Ok(s) => s,
        Err(e) => {
            set_last_error("Invalid session ID".to_string());
            return e as c_int;
        }
    };

    let _token = if !payment_token.is_null() {
        cstr_to_string(payment_token).unwrap_or_default()
    } else {
        String::new()
    };

    let mut sessions = match SESSIONS.write() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Failed to write sessions".to_string());
            return AcpErrorCode::InternalError as c_int;
        }
    };

    let session = match sessions.get_mut(&id) {
        Some(s) => s,
        None => {
            set_last_error(format!("Session not found: {}", id));
            return AcpErrorCode::NotFound as c_int;
        }
    };

    if session.status == "completed" {
        set_last_error("Session already completed".to_string());
        return AcpErrorCode::InvalidOperation as c_int;
    }

    if session.status == "canceled" {
        set_last_error("Cannot complete canceled session".to_string());
        return AcpErrorCode::InvalidOperation as c_int;
    }

    // Complete the session
    session.status = "completed".to_string();
    session.updated_at = chrono::Utc::now().to_rfc3339();

    let order = OrderInternal {
        id: Uuid::new_v4().to_string(),
        checkout_session_id: session.id.clone(),
        status: "placed".to_string(),
        permalink_url: Some(format!("https://orders.example.com/{}", session.id)),
    };

    let ffi_session = session_to_ffi(session);
    let ffi_order = AcpOrder {
        id: str_to_cstring(&order.id),
        checkout_session_id: str_to_cstring(&order.checkout_session_id),
        status: order_status_from_str(&order.status),
        permalink_url: order.permalink_url.as_ref().map(|s| str_to_cstring(s)).unwrap_or(ptr::null_mut()),
    };

    unsafe {
        (*out_result).session = ffi_session;
        (*out_result).order = ffi_order;
    }

    AcpErrorCode::Success as c_int
}

/// Cancel a checkout session.
#[no_mangle]
pub extern "C" fn acp_cancel_checkout_session(
    session_id: *const c_char,
    out_session: *mut AcpCheckoutSession,
) -> c_int {
    if session_id.is_null() || out_session.is_null() {
        set_last_error("Null pointer provided".to_string());
        return AcpErrorCode::NullPointer as c_int;
    }

    let id = match cstr_to_string(session_id) {
        Ok(s) => s,
        Err(e) => {
            set_last_error("Invalid session ID".to_string());
            return e as c_int;
        }
    };

    let mut sessions = match SESSIONS.write() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Failed to write sessions".to_string());
            return AcpErrorCode::InternalError as c_int;
        }
    };

    let session = match sessions.get_mut(&id) {
        Some(s) => s,
        None => {
            set_last_error(format!("Session not found: {}", id));
            return AcpErrorCode::NotFound as c_int;
        }
    };

    if session.status == "completed" {
        set_last_error("Cannot cancel completed session".to_string());
        return AcpErrorCode::InvalidOperation as c_int;
    }

    session.status = "canceled".to_string();
    session.updated_at = chrono::Utc::now().to_rfc3339();

    let ffi_session = session_to_ffi(session);
    unsafe {
        *out_session = ffi_session;
    }

    AcpErrorCode::Success as c_int
}

/// Free a checkout session allocated by this library.
#[no_mangle]
pub extern "C" fn acp_free_checkout_session(session: *mut AcpCheckoutSession) {
    if session.is_null() {
        return;
    }

    unsafe {
        let s = &mut *session;

        acp_free_string(s.id);
        acp_free_string(s.created_at);
        acp_free_string(s.updated_at);
        acp_free_string(s.totals.subtotal_currency);
        acp_free_string(s.totals.tax_currency);
        acp_free_string(s.totals.shipping_currency);
        acp_free_string(s.totals.discount_currency);
        acp_free_string(s.totals.grand_total_currency);

        if !s.items.is_null() && s.items_count > 0 {
            let items = std::slice::from_raw_parts_mut(s.items, s.items_count);
            for item in items {
                acp_free_string(item.id);
                acp_free_string(item.title);
                acp_free_string(item.unit_price_currency);
                acp_free_string(item.variant_id);
                acp_free_string(item.sku);
                acp_free_string(item.image_url);
            }
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(s.items, s.items_count) as *mut [AcpLineItem]);
        }
    }
}

/// Free a checkout session with order allocated by this library.
#[no_mangle]
pub extern "C" fn acp_free_checkout_session_with_order(result: *mut AcpCheckoutSessionWithOrder) {
    if result.is_null() {
        return;
    }

    unsafe {
        let r = &mut *result;
        acp_free_checkout_session(&mut r.session);
        acp_free_string(r.order.id);
        acp_free_string(r.order.checkout_session_id);
        acp_free_string(r.order.permalink_url);
    }
}

/// Create a checkout session from JSON.
#[no_mangle]
pub extern "C" fn acp_create_checkout_session_json(
    json_request: *const c_char,
    out_json: *mut *mut c_char,
) -> c_int {
    if json_request.is_null() || out_json.is_null() {
        set_last_error("Null pointer provided".to_string());
        return AcpErrorCode::NullPointer as c_int;
    }

    let request_str = match cstr_to_string(json_request) {
        Ok(s) => s,
        Err(e) => return e as c_int,
    };

    #[derive(Deserialize)]
    struct JsonRequest {
        items: Vec<JsonItem>,
    }

    #[derive(Deserialize)]
    struct JsonItem {
        id: String,
        quantity: i32,
    }

    let request: JsonRequest = match serde_json::from_str(&request_str) {
        Ok(r) => r,
        Err(e) => {
            set_last_error(format!("JSON parse error: {}", e));
            return AcpErrorCode::JsonError as c_int;
        }
    };

    let mut line_items = Vec::new();
    for item in request.items {
        if let Some(mut product) = get_product(&item.id) {
            product.quantity = item.quantity;
            line_items.push(product);
        } else {
            set_last_error(format!("Product not found: {}", item.id));
            return AcpErrorCode::NotFound as c_int;
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    let totals = calculate_totals(&line_items);

    let session = CheckoutSessionInternal {
        id: Uuid::new_v4().to_string(),
        status: "not_ready_for_payment".to_string(),
        items: line_items,
        totals,
        created_at: now.clone(),
        updated_at: now,
    };

    // Store session
    if let Ok(mut sessions) = SESSIONS.write() {
        sessions.insert(session.id.clone(), session.clone());
    }

    let json_response = match serde_json::to_string(&session) {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("JSON serialize error: {}", e));
            return AcpErrorCode::JsonError as c_int;
        }
    };

    unsafe {
        *out_json = str_to_cstring(&json_response);
    }

    AcpErrorCode::Success as c_int
}

/// Get a checkout session as JSON.
#[no_mangle]
pub extern "C" fn acp_get_checkout_session_json(
    session_id: *const c_char,
    out_json: *mut *mut c_char,
) -> c_int {
    if session_id.is_null() || out_json.is_null() {
        set_last_error("Null pointer provided".to_string());
        return AcpErrorCode::NullPointer as c_int;
    }

    let id = match cstr_to_string(session_id) {
        Ok(s) => s,
        Err(e) => return e as c_int,
    };

    let sessions = match SESSIONS.read() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Failed to read sessions".to_string());
            return AcpErrorCode::InternalError as c_int;
        }
    };

    match sessions.get(&id) {
        Some(session) => {
            let json_response = match serde_json::to_string(session) {
                Ok(s) => s,
                Err(e) => {
                    set_last_error(format!("JSON serialize error: {}", e));
                    return AcpErrorCode::JsonError as c_int;
                }
            };

            unsafe {
                *out_json = str_to_cstring(&json_response);
            }
            AcpErrorCode::Success as c_int
        }
        None => {
            set_last_error(format!("Session not found: {}", id));
            AcpErrorCode::NotFound as c_int
        }
    }
}

/// Get library version.
#[no_mangle]
pub extern "C" fn acp_version() -> *const c_char {
    static VERSION: &str = "1.0.0\0";
    VERSION.as_ptr() as *const c_char
}
