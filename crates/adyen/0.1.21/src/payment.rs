use super::action::Action;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalData {
    pub card_holder_name: String,
    pub issuer_country: String,
    pub card_summary: String,
    pub expiry_date: String,    // The expiry date on the card (M/yyyy).
    pub payment_method: String, // visa, mastercard, etc.

    #[serde(rename = "recurring.recurringDetailReference")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub recurring_detail_reference: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "resultCode")]
pub enum Response {
    /// The payment has been successfully authenticated with 3D Secure 2. Returned for 3D Secure 2
    /// authentication-only transactions.
    #[serde(rename_all = "camelCase")]
    AuthenticationFinished {},

    /// The transaction does not require 3D Secure authentication. Returned for standalone
    /// authentication-only integrations (cf.
    /// https://docs.adyen.com/online-payments/3d-secure/other-3ds-flows/authentication-only).
    #[serde(rename_all = "camelCase")]
    AuthenticationNotRequired {},

    /// The payment was successfully authorised. This state serves as an indicator to proceed with
    /// the delivery of goods and services. This is a final state.
    #[serde(rename_all = "camelCase")]
    Authorised {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        additional_data: Option<AdditionalData>,

        psp_reference: String,

        merchant_reference: String,
    },

    /// Indicates the payment has been cancelled (either by the shopper or the merchant) before
    /// processing was completed. This is a final state.
    #[serde(rename_all = "camelCase")]
    Cancelled {
        #[serde(rename = "refusalReasonCode")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        refusal_reason: Option<RefusalReason>,

        psp_reference: String,
    },

    /// The issuer requires further shopper interaction before the payment can be authenticated.
    /// Returned for 3D Secure 2 transactions.
    ChallengeShopper { action: Action },

    /// There was an error when the payment was being processed. The reason is given in the
    /// refusalReason field. This is a final state.
    #[serde(rename_all = "camelCase")]
    Error {
        #[serde(rename = "refusalReasonCode")]
        refusal_reason: RefusalReason,

        psp_reference: String,
    },

    /// The issuer requires the shopper's device fingerprint before the payment can be
    /// authenticated. Returned for 3D Secure 2 transactions.
    #[serde(rename_all = "camelCase")]
    IdentifyShopper { action: Action },

    /// The payment has been authorised for a partial amount. This happens for card payments when
    /// the merchant supports Partial Authorisations and the cardholder has insufficient funds.
    #[serde(rename_all = "camelCase")]
    PartiallyAuthorised {},

    /// Indicates that it is not possible to obtain the final status of the payment. This can
    /// happen if the systems providing final status information for the payment are unavailable,
    /// or if the shopper needs to take further action to complete the payment.
    #[serde(rename_all = "camelCase")]
    Pending { action: Action },

    /// Indicates that the response contains additional information that you need to present to a
    /// shopper, so that they can use it to complete a payment.
    #[serde(rename_all = "camelCase")]
    PresentToShopper {},

    /// Indicates the payment has successfully been received by Adyen, and will be processed. This
    /// is the initial state for all payments.
    #[serde(rename_all = "camelCase")]
    Received {},

    /// Indicates the shopper should be redirected to an external web page or app to complete the
    /// authorisation.
    #[serde(rename_all = "camelCase")]
    RedirectShopper { action: Action },

    /// Indicates the payment was refused. The reason is given in the refusalReason field. This is
    /// a final state.
    #[serde(rename_all = "camelCase")]
    Refused {
        #[serde(rename = "refusalReasonCode")]
        refusal_reason: RefusalReason,

        psp_reference: String,
    },
}

/// Represents various reasons why a transaction might be refused.
/// https://docs.adyen.com/development-resources/refusal-reasons/
#[derive(Clone, Debug, PartialEq)]
pub enum RefusalReason {
    /// The transaction was outright refused.
    Refused,

    /// The transaction needs a referral.
    Referral,

    /// An error occurred on the acquirer's end.
    AcquirerError,

    /// The card used is blocked and cannot be used for transactions.
    BlockedCard,

    /// The card has expired.
    ExpiredCard,

    /// The amount provided in the transaction does not match the expected amount.
    InvalidAmount,

    /// The card number provided is invalid.
    InvalidCardNumber,

    /// Could not contact the issuer to authorize the transaction.
    IssuerUnavailable,

    /// The type of transaction is not supported by the shopper's bank.
    NotSupported,

    /// 3D Secure authentication was not performed or failed.
    ThreeDNotAuthenticated,

    /// Insufficient funds in the account to cover the transaction.
    NotEnoughBalance,

    /// Possible fraud detected by the acquirer.
    AcquirerFraud,

    /// The transaction was cancelled by the system.
    Cancelled,

    /// The transaction was cancelled by the shopper.
    ShopperCancelled,

    /// The PIN entered is invalid.
    InvalidPin,

    /// The PIN was entered incorrectly too many times.
    PinTriesExceeded,

    /// The PIN could not be validated.
    PinValidationNotPossible,

    /// The transaction was flagged as fraudulent due to risk checks.
    Fraud,

    /// The transaction was not submitted correctly for processing.
    NotSubmitted,

    /// The transaction was flagged as fraudulent after both pre and post authorization checks.
    FraudCancelled,

    /// The transaction is not permitted based on various conditions.
    TransactionNotPermitted,

    /// The Card Verification Code (CVC) was declined.
    CvcDeclined,

    /// The card has restrictions or is invalid for the transaction context.
    RestrictedCard,

    /// The authorization for recurring transactions was revoked.
    RevocationOfAuth,

    /// A generic decline where the specific reason cannot be mapped accurately.
    DeclinedNonGeneric,

    /// The amount withdrawn exceeds the card's limit.
    WithdrawalAmountExceeded,

    /// The number of withdrawals exceeds the card's limit.
    WithdrawalCountExceeded,

    /// The issuer suspects fraud in the transaction.
    IssuerSuspectedFraud,

    /// Address Verification System (AVS) failed.
    AvsDeclined,

    /// The card requires an online PIN for the transaction.
    CardRequiresOnlinePin,

    /// The card does not have a checking account linked.
    NoCheckingAccountAvailable,

    /// The card does not have a savings account linked.
    NoSavingsAccountAvailable,

    /// A mobile PIN is required for the transaction.
    MobilePinRequired,

    /// The shopper abandoned the transaction after a contactless payment attempt failed.
    ContactlessFallback,

    /// Authentication is required for the transaction.
    AuthenticationRequired,

    /// The RReq was not received from DS during 3D Secure flow.
    RReqNotReceived,

    /// The current AID (Application Identifier) is in the penalty box.
    CurrentAidInPenaltyBox,

    /// Card Verification Method (CVM) like PIN or signature is required.
    CvmRequiredRestartPayment,

    /// An error occurred during 3D Secure authentication.
    ThreeDsAuthenticationError,

    /// Transaction blocked by Adyen to prevent excessive retry fees.
    TransactionBlockedByAdyen,
}

impl Serialize for RefusalReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let code = match self {
            RefusalReason::Refused => "2",
            RefusalReason::Referral => "3",
            RefusalReason::AcquirerError => "4",
            RefusalReason::BlockedCard => "5",
            RefusalReason::ExpiredCard => "6",
            RefusalReason::InvalidAmount => "7",
            RefusalReason::InvalidCardNumber => "8",
            RefusalReason::IssuerUnavailable => "9",
            RefusalReason::NotSupported => "10",
            RefusalReason::ThreeDNotAuthenticated => "11",
            RefusalReason::NotEnoughBalance => "12",
            RefusalReason::AcquirerFraud => "14",
            RefusalReason::Cancelled => "15",
            RefusalReason::ShopperCancelled => "16",
            RefusalReason::InvalidPin => "17",
            RefusalReason::PinTriesExceeded => "18",
            RefusalReason::PinValidationNotPossible => "19",
            RefusalReason::Fraud => "20",
            RefusalReason::NotSubmitted => "21",
            RefusalReason::FraudCancelled => "22",
            RefusalReason::TransactionNotPermitted => "23",
            RefusalReason::CvcDeclined => "24",
            RefusalReason::RestrictedCard => "25",
            RefusalReason::RevocationOfAuth => "26",
            RefusalReason::DeclinedNonGeneric => "27",
            RefusalReason::WithdrawalAmountExceeded => "28",
            RefusalReason::WithdrawalCountExceeded => "29",
            RefusalReason::IssuerSuspectedFraud => "31",
            RefusalReason::AvsDeclined => "32",
            RefusalReason::CardRequiresOnlinePin => "33",
            RefusalReason::NoCheckingAccountAvailable => "34",
            RefusalReason::NoSavingsAccountAvailable => "35",
            RefusalReason::MobilePinRequired => "36",
            RefusalReason::ContactlessFallback => "37",
            RefusalReason::AuthenticationRequired => "38",
            RefusalReason::RReqNotReceived => "39",
            RefusalReason::CurrentAidInPenaltyBox => "40",
            RefusalReason::CvmRequiredRestartPayment => "41",
            RefusalReason::ThreeDsAuthenticationError => "42",
            RefusalReason::TransactionBlockedByAdyen => "46",
        };
        serializer.serialize_str(code)
    }
}

impl<'de> Deserialize<'de> for RefusalReason {
    fn deserialize<D>(deserializer: D) -> Result<RefusalReason, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RefusalReasonVisitor;

        impl<'de> serde::de::Visitor<'de> for RefusalReasonVisitor {
            type Value = RefusalReason;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a refusal reason code")
            }

            fn visit_str<E>(self, value: &str) -> Result<RefusalReason, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "2" => Ok(RefusalReason::Refused),
                    "3" => Ok(RefusalReason::Referral),
                    "4" => Ok(RefusalReason::AcquirerError),
                    "5" => Ok(RefusalReason::BlockedCard),
                    "6" => Ok(RefusalReason::ExpiredCard),
                    "7" => Ok(RefusalReason::InvalidAmount),
                    "8" => Ok(RefusalReason::InvalidCardNumber),
                    "9" => Ok(RefusalReason::IssuerUnavailable),
                    "10" => Ok(RefusalReason::NotSupported),
                    "11" => Ok(RefusalReason::ThreeDNotAuthenticated),
                    "12" => Ok(RefusalReason::NotEnoughBalance),
                    "14" => Ok(RefusalReason::AcquirerFraud),
                    "15" => Ok(RefusalReason::Cancelled),
                    "16" => Ok(RefusalReason::ShopperCancelled),
                    "17" => Ok(RefusalReason::InvalidPin),
                    "18" => Ok(RefusalReason::PinTriesExceeded),
                    "19" => Ok(RefusalReason::PinValidationNotPossible),
                    "20" => Ok(RefusalReason::Fraud),
                    "21" => Ok(RefusalReason::NotSubmitted),
                    "22" => Ok(RefusalReason::FraudCancelled),
                    "23" => Ok(RefusalReason::TransactionNotPermitted),
                    "24" => Ok(RefusalReason::CvcDeclined),
                    "25" => Ok(RefusalReason::RestrictedCard),
                    "26" => Ok(RefusalReason::RevocationOfAuth),
                    "27" => Ok(RefusalReason::DeclinedNonGeneric),
                    "28" => Ok(RefusalReason::WithdrawalAmountExceeded),
                    "29" => Ok(RefusalReason::WithdrawalCountExceeded),
                    "31" => Ok(RefusalReason::IssuerSuspectedFraud),
                    "32" => Ok(RefusalReason::AvsDeclined),
                    "33" => Ok(RefusalReason::CardRequiresOnlinePin),
                    "34" => Ok(RefusalReason::NoCheckingAccountAvailable),
                    "35" => Ok(RefusalReason::NoSavingsAccountAvailable),
                    "36" => Ok(RefusalReason::MobilePinRequired),
                    "37" => Ok(RefusalReason::ContactlessFallback),
                    "38" => Ok(RefusalReason::AuthenticationRequired),
                    "39" => Ok(RefusalReason::RReqNotReceived),
                    "40" => Ok(RefusalReason::CurrentAidInPenaltyBox),
                    "41" => Ok(RefusalReason::CvmRequiredRestartPayment),
                    "42" => Ok(RefusalReason::ThreeDsAuthenticationError),
                    "46" => Ok(RefusalReason::TransactionBlockedByAdyen),
                    _ => Err(serde::de::Error::unknown_variant(
                        value,
                        &[
                            "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "14", "15",
                            "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27",
                            "28", "29", "31", "32", "33", "34", "35", "36", "37", "38", "39", "40",
                            "41", "42", "46",
                        ],
                    )),
                }
            }
        }

        deserializer.deserialize_str(RefusalReasonVisitor)
    }
}
