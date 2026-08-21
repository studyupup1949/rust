use serde::{Deserialize, Serialize};
use crate::Amount;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalData {
    pub hmac_signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "eventCode")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationRequestItem {
    // The success field informs you of the outcome of a payment request.
    #[serde(rename_all = "camelCase")]
    Authorisation {
        aditional_data: AdditionalData,
        success: String,
        event_date: String,
        merchant_account_code: String,
        psp_reference: String,
        merchant_reference: String,
        amount: Amount,
    }, 
    
    // The success field informs you of the outcome of a request to adjust the authorised amount.
    #[serde(rename_all = "camelCase")]
    AuthorisationAdjustment {}, 
    
    // The success field informs you of the outcome of a request to cancel a payment.
    #[serde(rename_all = "camelCase")]
    Cancellation {}, 
    
    // The success field informs you of the outcome of a request to cancel or refund a payment.
    #[serde(rename_all = "camelCase")]
    CancelOrRefund {}, 
    
    // The success field informs you of the outcome of a request to capture a payment.
    #[serde(rename_all = "camelCase")]
    Capture {}, 
    
    // The capture failed due to rejection by the card scheme.
    #[serde(rename_all = "camelCase")]
    CaptureFailed {}, 
    
    // The original payment has expired on the Adyen payments platform.
    #[serde(rename_all = "camelCase")]
    Expire {}, 
    
    // The payment has been handled outside the Adyen payments platform.
    #[serde(rename_all = "camelCase")]
    HandledExternally {}, 
    
    // Sent when the first payment for your payment request is a partial payment, and an order has been created.
    #[serde(rename_all = "camelCase")]
    OrderOpened {}, 
    
    // The success field informs you of the outcome of the shopper's last payment when paying for an order in partial payments.
    #[serde(rename_all = "camelCase")]
    OrderClosed {}, 
    
    // The success field informs you of the outcome of a request to refund a payment.
    #[serde(rename_all = "camelCase")]
    Refund {}, 
    
    // The refund failed due to a rejection by the card scheme.
    #[serde(rename_all = "camelCase")]
    RefundFailed {}, 
    
    // The refunded amount has been returned to Adyen, and is back in your account.
    #[serde(rename_all = "camelCase")]
    RefundedReversed {}, 
    
    // The success field informs you of the outcome of a request to refund with data.
    #[serde(rename_all = "camelCase")]
    RefundWithData {}, 
    
    // A new report is available.
    #[serde(rename_all = "camelCase")]
    ReportAvailable {}, 
    
    // The success field informs you of the outcome of a request to cancel an unreferenced POS refund.
    #[serde(rename_all = "camelCase")]
    VoidPendingRefund {}, 
    
    // A payment was charged back, and the funds were deducted from your account.
    #[serde(rename_all = "camelCase")]
    Chargeback {}, 
    
    // A chargeback has been defended towards the issuing bank.
    #[serde(rename_all = "camelCase")]
    ChargebackReversed {}, 
    
    // The dispute process has opened.
    #[serde(rename_all = "camelCase")]
    NotificationOfChargeback {}, 
    
    // The alert passed on by issuers to schemes and subsequently to processors.
    #[serde(rename_all = "camelCase")]
    NotificationOfFraud {}, 
    
    // Your pre-arbitration case has been declined by the cardholder's bank.
    #[serde(rename_all = "camelCase")]
    PrearbitrationLost {}, 
    
    // Your pre-arbitration case has been accepted by the cardholder's bank.
    #[serde(rename_all = "camelCase")]
    PrearbitrationWon {}, 
    
    // A shopper has opened an RFI (Request for Information) case with the bank.
    #[serde(rename_all = "camelCase")]
    RequestForInformation {}, 
    
    // The issuing bank declined the material submitted during defense of the original chargeback.
    #[serde(rename_all = "camelCase")]
    SecondChargeback {}, 
    
    // The payout has expired.
    #[serde(rename_all = "camelCase")]
    PayoutExpire {}, 
    
    // The user reviewing the payout declined it.
    #[serde(rename_all = "camelCase")]
    PayoutDecline {}, 
    
    // The success field informs you of the outcome of a payout request.
    #[serde(rename_all = "camelCase")]
    PayoutThirdparty {}, 
    
    // The financial institution rejected the payout.
    #[serde(rename_all = "camelCase")]
    PaidoutReversed {}, 
    
    // The offer has expired.
    #[serde(rename_all = "camelCase")]
    OfferClosed {}, 
    
    // A recurring contract has been created.
    #[serde(rename_all = "camelCase")]
    RecurringContract {}, 
    
    // The refund for the payment will be performed after the payment is captured.
    #[serde(rename_all = "camelCase")]
    PostponedRefund {}, 
    
    // An authentication-only flow was performed.
    #[serde(rename_all = "camelCase")]
    Authentication {}, 
    
    // The manual review triggered by risk rules was accepted.
    #[serde(rename_all = "camelCase")]
    ManualReviewAccept {}, 
    
    // The manual review triggered by risk rules was rejected.
    #[serde(rename_all = "camelCase")]
    ManualReviewReject {},
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub live: String,

    pub notification_items: Vec<NotificationRequestItem>,
}
