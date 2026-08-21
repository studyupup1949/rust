use anyhow::bail;
use cosmwasm_std::{Binary, IbcDstCallback, IbcSrcCallback};
use cw20_ics20::ibc::Ics20Packet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// This is copoied from cosmwasm-std because fields are private there
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IbcCallbackRequest {
    // using private fields to force use of the constructors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_callback: Option<IbcSrcCallback>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest_callback: Option<IbcDstCallback>,
}

pub fn parse_ics20_memo_callback(packet_data: &Binary) -> anyhow::Result<()> {
    if let Ok(packet) = cosmwasm_std::from_json::<Ics20Packet>(&packet_data) {
        if let Ok(callback_request) =
            serde_json::from_str::<IbcCallbackRequest>(&packet.memo.unwrap_or("{}".to_string()))
        {
            if callback_request.src_callback.is_some() {
                return Ok(());
            }
        }
    }

    bail!("No ics_20_memo callback")
}
