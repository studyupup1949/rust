use std::fmt;

#[derive(Debug, Clone)]
pub struct AbiInput {
    pub name: Option<String>,
    pub r#type: String,
    pub indexed: Option<bool>,
    #[allow(dead_code)]
    pub internal_type: Option<String>,
    #[allow(dead_code)]
    pub components: Option<Vec<AbiInput>>,
}

#[derive(Debug, Clone)]
pub struct AbiOutput {
    pub name: Option<String>,
    pub r#type: String,
    #[allow(dead_code)]
    pub internal_type: Option<String>,
    #[allow(dead_code)]
    pub components: Option<Vec<AbiOutput>>,
}

#[derive(Debug, Clone)]
pub struct AbiItem {
    pub r#type: String,
    pub name: Option<String>,
    pub inputs: Option<Vec<AbiInput>>,
    pub outputs: Option<Vec<AbiOutput>>,
    pub state_mutability: Option<String>,
    #[allow(dead_code)]
    pub anonymous: Option<bool>,
    #[allow(dead_code)]
    pub payable: Option<bool>,
    #[allow(dead_code)]
    pub constant: Option<bool>,
}

impl fmt::Display for AbiItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let visibility = self.state_mutability.as_deref().unwrap_or("nonpayable");
        let type_str = &self.r#type;

        match type_str.as_str() {
            "constructor" => {
                let params = self
                    .inputs
                    .as_ref()
                    .map(|inputs| {
                        inputs
                            .iter()
                            .map(|p| {
                                if let Some(name) = &p.name {
                                    format!("{} {}", p.r#type, name)
                                } else {
                                    p.r#type.clone()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                write!(f, "constructor({params})")
            }
            "event" => {
                let params = self
                    .inputs
                    .as_ref()
                    .map(|inputs| {
                        inputs
                            .iter()
                            .map(|p| {
                                let indexed = if p.indexed.unwrap_or(false) {
                                    " indexed"
                                } else {
                                    ""
                                };
                                if let Some(name) = &p.name {
                                    format!("{}{} {}", p.r#type, indexed, name)
                                } else {
                                    format!("{}{}", p.r#type, indexed)
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                write!(
                    f,
                    "event {}({})",
                    self.name.as_ref().unwrap_or(&String::new()),
                    params
                )
            }
            "function" => {
                let params = self
                    .inputs
                    .as_ref()
                    .map(|inputs| {
                        inputs
                            .iter()
                            .map(|p| {
                                if let Some(name) = &p.name {
                                    format!("{} {}", p.r#type, name)
                                } else {
                                    p.r#type.clone()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();

                let returns = if let Some(outputs) = &self.outputs {
                    if !outputs.is_empty() {
                        let output_params = outputs
                            .iter()
                            .map(|o| {
                                let has_name = o.name.as_ref().is_some_and(|n| !n.is_empty());
                                if has_name {
                                    format!("{} {}", o.r#type, o.name.as_ref().unwrap())
                                } else {
                                    o.r#type.clone()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!(" returns ({output_params})")
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let vis = if visibility != "nonpayable" {
                    format!(" {visibility}")
                } else {
                    String::new()
                };

                write!(
                    f,
                    "function {}({}){}{}",
                    self.name.as_ref().unwrap_or(&String::new()),
                    params,
                    vis,
                    returns
                )
            }
            "fallback" => {
                let payable = if visibility == "payable" {
                    " payable"
                } else {
                    ""
                };
                write!(f, "fallback() external{payable}")
            }
            "receive" => {
                write!(f, "receive() external payable")
            }
            _ => {
                write!(f, "unknown")
            }
        }
    }
}
