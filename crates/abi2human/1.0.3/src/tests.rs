#[cfg(test)]
use crate::abi::{AbiInput, AbiItem, AbiOutput};
#[cfg(test)]
use crate::converter::Converter;
#[cfg(test)]
use crate::json_parser::JsonParser;

#[test]
fn test_parse_simple_function() {
    let json = r#"[{
        "type": "function",
        "name": "transfer",
        "inputs": [
            {"name": "to", "type": "address"},
            {"name": "amount", "type": "uint256"}
        ],
        "outputs": [{"type": "bool"}],
        "stateMutability": "nonpayable"
    }]"#;

    let mut parser = JsonParser::new(json);
    let result = parser.parse_abi();
    assert!(result.is_ok());

    let items = result.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name.as_ref().unwrap(), "transfer");
}

#[test]
fn test_format_function() {
    let item = AbiItem {
        r#type: "function".to_string(),
        name: Some("transfer".to_string()),
        inputs: Some(vec![
            AbiInput {
                name: Some("to".to_string()),
                r#type: "address".to_string(),
                indexed: None,
                internal_type: None,
                components: None,
            },
            AbiInput {
                name: Some("amount".to_string()),
                r#type: "uint256".to_string(),
                indexed: None,
                internal_type: None,
                components: None,
            },
        ]),
        outputs: Some(vec![AbiOutput {
            name: None,
            r#type: "bool".to_string(),
            internal_type: None,
            components: None,
        }]),
        state_mutability: Some("nonpayable".to_string()),
        anonymous: None,
        payable: None,
        constant: None,
    };

    let formatted = item.to_string();
    assert_eq!(
        formatted,
        "function transfer(address to, uint256 amount) returns (bool)"
    );
}

#[test]
fn test_format_event() {
    let item = AbiItem {
        r#type: "event".to_string(),
        name: Some("Transfer".to_string()),
        inputs: Some(vec![
            AbiInput {
                name: Some("from".to_string()),
                r#type: "address".to_string(),
                indexed: Some(true),
                internal_type: None,
                components: None,
            },
            AbiInput {
                name: Some("to".to_string()),
                r#type: "address".to_string(),
                indexed: Some(true),
                internal_type: None,
                components: None,
            },
            AbiInput {
                name: Some("value".to_string()),
                r#type: "uint256".to_string(),
                indexed: Some(false),
                internal_type: None,
                components: None,
            },
        ]),
        outputs: None,
        state_mutability: None,
        anonymous: None,
        payable: None,
        constant: None,
    };

    let formatted = item.to_string();
    assert_eq!(
        formatted,
        "event Transfer(address indexed from, address indexed to, uint256 value)"
    );
}

#[test]
fn test_format_constructor() {
    let item = AbiItem {
        r#type: "constructor".to_string(),
        name: None,
        inputs: Some(vec![AbiInput {
            name: Some("initialSupply".to_string()),
            r#type: "uint256".to_string(),
            indexed: None,
            internal_type: None,
            components: None,
        }]),
        outputs: None,
        state_mutability: None,
        anonymous: None,
        payable: None,
        constant: None,
    };

    let formatted = item.to_string();
    assert_eq!(formatted, "constructor(uint256 initialSupply)");
}

#[test]
fn test_format_fallback() {
    let item = AbiItem {
        r#type: "fallback".to_string(),
        name: None,
        inputs: None,
        outputs: None,
        state_mutability: Some("payable".to_string()),
        anonymous: None,
        payable: None,
        constant: None,
    };

    let formatted = item.to_string();
    assert_eq!(formatted, "fallback() external payable");
}

#[test]
fn test_format_receive() {
    let item = AbiItem {
        r#type: "receive".to_string(),
        name: None,
        inputs: None,
        outputs: None,
        state_mutability: None,
        anonymous: None,
        payable: None,
        constant: None,
    };

    let formatted = item.to_string();
    assert_eq!(formatted, "receive() external payable");
}

#[test]
fn test_parse_with_abi_wrapper() {
    let json = r#"{
        "abi": [
            {
                "type": "function",
                "name": "balanceOf",
                "inputs": [{"name": "account", "type": "address"}],
                "outputs": [{"type": "uint256"}],
                "stateMutability": "view"
            }
        ]
    }"#;

    let mut parser = JsonParser::new(json);
    let result = parser.parse_abi();
    assert!(result.is_ok());

    let items = result.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name.as_ref().unwrap(), "balanceOf");
}

#[test]
fn test_convert_to_human_readable() {
    let items = vec![AbiItem {
        r#type: "function".to_string(),
        name: Some("approve".to_string()),
        inputs: Some(vec![
            AbiInput {
                name: Some("spender".to_string()),
                r#type: "address".to_string(),
                indexed: None,
                internal_type: None,
                components: None,
            },
            AbiInput {
                name: Some("value".to_string()),
                r#type: "uint256".to_string(),
                indexed: None,
                internal_type: None,
                components: None,
            },
        ]),
        outputs: Some(vec![AbiOutput {
            name: None,
            r#type: "bool".to_string(),
            internal_type: None,
            components: None,
        }]),
        state_mutability: Some("nonpayable".to_string()),
        anonymous: None,
        payable: None,
        constant: None,
    }];

    let readable = Converter::convert_to_human_readable(&items);
    assert_eq!(readable.len(), 1);
    assert_eq!(
        readable[0],
        "function approve(address spender, uint256 value) returns (bool)"
    );
}

#[test]
fn test_format_json_array() {
    let items = vec![
        "function transfer(address to, uint256 amount) returns (bool)".to_string(),
        "event Transfer(address indexed from, address indexed to, uint256 value)".to_string(),
    ];

    let formatted = Converter::format_as_json_array(&items, false);
    assert_eq!(
        formatted,
        r#"["function transfer(address to, uint256 amount) returns (bool)","event Transfer(address indexed from, address indexed to, uint256 value)"]"#
    );
}

#[test]
fn test_view_function() {
    let item = AbiItem {
        r#type: "function".to_string(),
        name: Some("balanceOf".to_string()),
        inputs: Some(vec![AbiInput {
            name: Some("account".to_string()),
            r#type: "address".to_string(),
            indexed: None,
            internal_type: None,
            components: None,
        }]),
        outputs: Some(vec![AbiOutput {
            name: None,
            r#type: "uint256".to_string(),
            internal_type: None,
            components: None,
        }]),
        state_mutability: Some("view".to_string()),
        anonymous: None,
        payable: None,
        constant: None,
    };

    let formatted = item.to_string();
    assert_eq!(
        formatted,
        "function balanceOf(address account) view returns (uint256)"
    );
}

#[test]
fn test_payable_function() {
    let item = AbiItem {
        r#type: "function".to_string(),
        name: Some("deposit".to_string()),
        inputs: Some(vec![]),
        outputs: None,
        state_mutability: Some("payable".to_string()),
        anonymous: None,
        payable: None,
        constant: None,
    };

    let formatted = item.to_string();
    assert_eq!(formatted, "function deposit() payable");
}
