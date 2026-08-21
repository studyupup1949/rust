use crate::{Address, Component};
use failure::Fail;
use failure::{format_err, Error};
use itertools::Itertools;
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;
use std::str::FromStr;
use strum::IntoEnumIterator;

const TEMPLATE_NAME: &'static str = "addr";

/// Represents a Regex and the value to replace the regex matches with
#[derive(Debug, Clone)]
pub(crate) struct Replacement {
    pub regex: regex::Regex,
    pub replacement_value: String,
}

/// Replacement rule
/// a Replacement can be on all fields, or only one of them
#[derive(Debug, Clone)]
pub(crate) enum ReplaceRule {
    All(Replacement),
    Component((Component, Replacement)),
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct CountryCode(String); // TODO small string

impl FromStr for CountryCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 2 {
            if s == "UK" {
                Ok(CountryCode("GB".to_owned()))
            } else {
                Ok(CountryCode(s.to_uppercase()))
            }
        } else {
            Err(format_err!(
                "{} is not a valid ISO3166-1:alpha2 country code",
                s,
            ))
        }
    }
}

impl CountryCode {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a new field to add the an address
#[derive(Debug, Clone)]
pub(crate) struct NewComponent {
    pub component: Component,
    pub new_value: String,
}

/// The template handle the handlerbar template used to format an [`Address`](struct.Address.html)
#[derive(Debug, Default)]
pub(crate) struct Template {
    /// Moustache template
    address_template: String, // used only to clone the template
    pub handlebar_handler: handlebars::Handlebars,
}

impl Template {
    pub fn new(address_template: &str) -> Self {
        let mut template_engine = crate::handlebar_helper::new_template_engine();
        template_engine
            .register_template_string(TEMPLATE_NAME, address_template)
            .expect("impossible to build template");
        Template {
            address_template: address_template.to_owned(),
            handlebar_handler: template_engine,
        }
    }
}

impl Clone for Template {
    fn clone(&self) -> Self {
        Self::new(self.address_template.as_str())
    }
}

/// The `Rules` contains all the rules used to cleanup the addresses
/// Some of those rules are used as preformating rules (before changing the [`Address`](struct.Address.html)
/// to a text with the handlebar template)
/// And some of those rules are used as postformating rules, on the formatted text
#[derive(Debug, Default, Clone)]
pub(crate) struct Rules {
    pub replace: Vec<ReplaceRule>,
    pub postformat_replace: Vec<Replacement>,
    pub change_country: Option<String>,
    pub change_country_code: Option<String>,
    /// Override the country
    pub add_component: Option<NewComponent>,
}

#[derive(Debug)]
pub(crate) struct Templates {
    pub default_template: Template,
    pub fallback_template: Template,
    pub templates_by_country: HashMap<CountryCode, Template>,
    pub rules_by_country: HashMap<CountryCode, Rules>,
    pub fallback_templates_by_country: HashMap<CountryCode, Template>,
    pub fallback_rules: Rules,
}

/// This [`Formatter`](struct.Formatter.html) holds all the configuration needed to format an [`Address`](struct.Address.html)
/// to a nice text.
///
/// The main method is the `format` method, that takes an [`Address`](struct.Address.html)
/// or something that can be converted to an [`Address`](struct.Address.html) and return a result with the formatted `String`
///
/// ```
/// # #[macro_use] extern crate maplit;
/// # fn main() {
///    use address_formatter::Component::*;
///    let formatter = address_formatter::Formatter::default();
///
///    let addr: address_formatter::Address = hashmap!(
///        City => "Toulouse",
///        Country => "France",
///        CountryCode => "FR",
///        County => "Toulouse",
///        HouseNumber => "17",
///        Neighbourhood => "Lafourguette",
///        Postcode => "31000",
///        Road => "Rue du Médecin-Colonel Calbairac",
///        State => "Midi-Pyrénées",
///        Suburb => "Toulouse Ouest",
///    ).into();
///
///    assert_eq!(
///        formatter.format(addr).unwrap(),
///        r#"17 Rue du Médecin-Colonel Calbairac
///31000 Toulouse
///France
///"#
///        .to_owned()
///    )
/// # }
///
/// ```
pub struct Formatter {
    pub(crate) templates: Templates,
    pub(crate) county_codes: HashMap<(CountryCode, String), String>,
    pub(crate) state_codes: HashMap<(CountryCode, String), String>,
    // country_to_lang: Vec<>,
    // abbreviations: Vec<>,
    // valid_replacement_components: Vec<>
}

/// This configuration changes the [`Formatter`](struct.Formatter.html) behavior
#[derive(Default, Debug)]
pub struct Configuration {
    /// force the use of a give country (so the [`Address`](struct.Address.html) country_code is not used)
    pub country_code: Option<String>,
    /// use abbreviation in the formated text (like "Avenue" to "Av.")
    pub abbreviate: Option<bool>,
}

impl Default for Formatter {
    /// Default constructor
    fn default() -> Self {
        crate::read_configuration::read_configuration()
    }
}

impl Formatter {
    /// make a human readable text from an [`Address`](struct.Address.html)
    /// ```
    /// # #[macro_use] extern crate maplit;
    /// # fn main() {
    ///    use address_formatter::Component::*;
    ///    let formatter = address_formatter::Formatter::default();
    ///
    ///    let addr: address_formatter::Address = hashmap!(
    ///        City => "Toulouse",
    ///        Country => "France",
    ///        CountryCode => "FR",
    ///        County => "Toulouse",
    ///        HouseNumber => "17",
    ///        Neighbourhood => "Lafourguette",
    ///        Postcode => "31000",
    ///        Road => "Rue du Médecin-Colonel Calbairac",
    ///        State => "Midi-Pyrénées",
    ///        Suburb => "Toulouse Ouest",
    ///    ).into();
    ///
    ///    assert_eq!(
    ///        formatter.format(addr).unwrap(),
    ///        r#"17 Rue du Médecin-Colonel Calbairac
    ///31000 Toulouse
    ///France
    ///"#
    ///        .to_owned()
    ///    )
    /// # }
    /// ```
    pub fn format(&self, into_addr: impl Into<Address>) -> Result<String, Error> {
        self.format_with_config(into_addr.into(), Configuration::default())
    }

    /// make a human readable text from an [`Address`](struct.Address.html)
    /// Same as the format method, but with a [`Configuration`](address_formatter::formatter::Configuration) object
    pub fn format_with_config(
        &self,
        into_addr: impl Into<Address>,
        conf: Configuration,
    ) -> Result<String, Error> {
        let mut addr = into_addr.into();
        let country_code = self.find_country_code(&mut addr, conf);

        sanity_clean_address(&mut addr);

        let template = self.find_template(&addr, &country_code);
        let rules = country_code
            .as_ref()
            .and_then(|c| self.templates.rules_by_country.get(c))
            .unwrap_or_else(|| &self.templates.fallback_rules);

        self.preformat(&rules, &mut addr);

        let text = template
            .handlebar_handler
            .render(TEMPLATE_NAME, &addr)
            .map_err(|e| e.context("impossible to render template"))?;

        let text = cleanup_rendered(&text, &rules);

        Ok(text)
    }

    fn find_country_code(&self, addr: &mut Address, conf: Configuration) -> Option<CountryCode> {
        let mut country_code = conf
            .country_code
            .or_else(|| addr[Component::CountryCode].clone())
            .and_then(|s| {
                CountryCode::from_str(&s)
                    .map_err(|e| log::info!("impossible to find a country: {}", e))
                    .ok()
            });

        // we hardcode some country code values
        if country_code == CountryCode::from_str("NL").ok() {
            if let Some(state) = addr[Component::State].clone() {
                if state.as_str() == "Curaçao" {
                    country_code = CountryCode::from_str("CW").ok();
                    addr[Component::Country] = Some("Curaçao".to_owned());
                }
                let state = state.to_lowercase();

                if state.as_str() == "sint maarten" {
                    country_code = CountryCode::from_str("SX").ok();
                    addr[Component::Country] = Some("Sint Maarten".to_owned());
                } else if state.as_str() == "aruba" {
                    country_code = CountryCode::from_str("AW").ok();
                    addr[Component::Country] = Some("Aruba".to_owned());
                }
            }
        }

        country_code
    }

    fn find_template<'a>(
        &'a self,
        addr: &Address,
        country_code: &Option<CountryCode>,
    ) -> &'a Template {
        country_code
            .as_ref()
            .and_then(|c| {
                if !has_minimum_address_components(addr) {
                    // if the address does not have the minimum fields, we get its country fallback template
                    // if there is a specidif one, else we get the default fallback template
                    self.templates
                        .fallback_templates_by_country
                        .get(&c)
                        .or_else(|| Some(&self.templates.fallback_template))
                } else {
                    self.templates.templates_by_country.get(&c)
                }
            })
            .unwrap_or(&self.templates.default_template)
    }

    fn preformat(&self, rules: &Rules, addr: &mut Address) {
        for r in &rules.replace {
            r.replace_fields(addr);
        }

        // in some cases, we need to add some components
        if let Some(add_component) = &rules.add_component {
            addr[add_component.component] = Some(add_component.new_value.clone());
        }
        if let Some(change_country) = &rules.change_country {
            addr[Component::Country] = Some(change_country.clone());
        }
        if let Some(change_country_code) = &rules.change_country_code {
            addr[Component::CountryCode] = Some(change_country_code.clone());
        }

        // we also try to find the state_code/county_code
        if let Some(country) = addr[Component::CountryCode]
            .as_ref()
            .and_then(|c| CountryCode::from_str(c).ok())
        {
            if addr[Component::StateCode].is_none() {
                // we try to see if we can use the state_code and the reference table 'state_codes.yaml' to find the state
                if let Some(state) = &addr[Component::State] {
                    if let Some(new_state) = self
                        .state_codes
                        .get(&(country.clone(), state.to_string()))
                        .cloned()
                    {
                        addr[Component::StateCode] = Some(new_state);
                    }
                }
            }

            if addr[Component::CountyCode].is_none() {
                // same for county
                if let Some(county) = &addr[Component::County] {
                    if let Some(new_county) = self
                        .county_codes
                        .get(&(country, county.to_string()))
                        .cloned()
                    {
                        addr[Component::County] = Some(new_county);
                    }
                }
            }
        }
    }
}

/// Build [`Address`](struct.Address.html) from a less structured input (like addresses from [Nominatim](https://github.com/openstreetmap/Nominatim))
///
/// It applies aliases rules to fill the [`Address`](struct.Address.html)'s fields as good as possible.
pub struct AddressBuilder {
    pub(crate) component_aliases: HashMap<Component, Vec<String>>,
}

impl Default for AddressBuilder {
    fn default() -> Self {
        crate::read_configuration::read_address_builder_configuration()
    }
}

impl AddressBuilder {
    /// Build an [`Address`](struct.Address.html)(crate::Address) from an unstructed source (like Nominatim output)
    pub fn build_address<'a>(
        &self,
        values: impl IntoIterator<Item = (&'a str, String)>,
    ) -> Address {
        let mut address = Address::default();
        let mut unknown = HashMap::<String, String>::new();
        for (k, v) in values.into_iter() {
            let component = Component::from_str(k).ok();;
            if let Some(component) = component {
                address[component] = Some(v);
            } else {
                unknown.insert(k.to_string(), v);
            }
        }

        // all the unknown fields are added in the 'Attention' field
        if !unknown.is_empty() {
            for (c, aliases) in &self.component_aliases {
                // if the address's component has not been already set, we set it to its first found alias
                for alias in aliases {
                    if let Some(a) = unknown.remove(alias) {
                        if address[*c].is_none() {
                            address[*c] = Some(a);
                        }
                    }
                }
            }
            address[Component::Attention] = Some(unknown.values().join(", "));
        }

        // hardocded cleanup for some bad country data
        if let (Some(state), Some(country)) =
            (&address[Component::State], &address[Component::Country])
        {
            if country.parse::<usize>().is_ok() {
                address[Component::Country] = Some(state.clone());
                address[Component::State] = None;
            }
        }
        address
    }
}

fn sanity_clean_address(addr: &mut Address) {
    lazy_static::lazy_static! {
        static ref POST_CODE_RANGE: Regex = Regex::new(r#"\d+;\d+"#).unwrap();
        static ref MATCHABLE_POST_CODE_RANGE: Regex = Regex::new(r#"^(\d{5}),\d{5}"#).unwrap();
        static ref IS_URL: Regex= Regex::new(r#"https?://"#).unwrap();

    }
    // cleanup the postcode
    if let Some(post_code) = &addr[Component::Postcode] {
        if post_code.len() > 20 || POST_CODE_RANGE.is_match(post_code) {
            addr[Component::Postcode] = None;
        } else if let Some(r) = MATCHABLE_POST_CODE_RANGE
            .captures(post_code)
            .and_then(|r| r.get(1))
            .map(|c| c.as_str())
        {
            addr[Component::Postcode] = Some(r.to_owned());
        }
    }

    // clean values containing URLs
    for c in Component::iter() {
        if let Some(v) = &addr[c] {
            if IS_URL.is_match(v) {
                addr[c] = None;
            }
        }
    }
}

fn cleanup_rendered(text: &str, rules: &Rules) -> String {
    use itertools::Itertools;
    lazy_static::lazy_static! {
        static ref REPLACEMENTS:  [(Regex, &'static str); 12]= [
            (RegexBuilder::new(r"[},\s]+$").multi_line(true).build().unwrap(), ""),
            (RegexBuilder::new(r"^ - ").multi_line(true).build().unwrap(), ""), // line starting with dash due to a parameter missing
            (RegexBuilder::new(r"^[,\s]+").multi_line(true).build().unwrap(), ""),
            (RegexBuilder::new(r",\s*,").multi_line(true).build().unwrap(), ", "), //multiple commas to one
            (RegexBuilder::new(r"[\t\p{Zs}]+,[\t\p{Zs}]+").multi_line(true).build().unwrap(), ", "), //one horiz whitespace behind comma
            (RegexBuilder::new(r"[\t ][\t ]+").multi_line(true).build().unwrap(), " "), //multiple horiz whitespace to one
            (RegexBuilder::new(r"[\t\p{Zs}]\n").multi_line(true).build().unwrap(), "\n"), //horiz whitespace, newline to newline
            (RegexBuilder::new(r"\n,").multi_line(true).build().unwrap(), "\n"), //newline comma to just newline
            (RegexBuilder::new(r",,+").multi_line(true).build().unwrap(), ","), //multiple commas to one
            (RegexBuilder::new(r",\n").multi_line(true).build().unwrap(), "\n"), //comma newline to just newline
            (RegexBuilder::new(r"\n[\t\p{Zs}]+").multi_line(true).build().unwrap(), "\n"), //newline plus space to newline
            (RegexBuilder::new(r"\n\n+").multi_line(true).build().unwrap(), "\n"), //multiple newline to one
        ];

        static ref FINAL_CLEANUP:  [(Regex, &'static str); 2]= [
            (Regex::new(r"^\s+").unwrap(), ""), //remove leading whitespace
            (Regex::new(r"\s+$").unwrap(), ""), //remove end whitespace
        ];
    }

    let mut res = text.to_owned();

    for (rgx, new_val) in REPLACEMENTS.iter() {
        let rep = rgx.replace_all(&res, *new_val);
        // to improve performance, we update the string only if it was changed by the replace
        match rep {
            std::borrow::Cow::Borrowed(_) => {}
            std::borrow::Cow::Owned(v) => {
                res = v;
            }
        }
    }

    for r in &rules.postformat_replace {
        let rep = r.regex.replace_all(&res, r.replacement_value.as_str());
        match rep {
            std::borrow::Cow::Borrowed(_) => {}
            std::borrow::Cow::Owned(v) => {
                res = v;
            }
        }
    }

    // we also dedup the string
    // we dedup and trim and all the same 'token' in a line
    // and all the same lines too
    let mut res = res
        .split('\n')
        .map(|s| s.split(", ").map(|e| e.trim()).dedup().join(", "))
        .dedup()
        .join("\n");

    for (rgx, new_val) in FINAL_CLEANUP.iter() {
        let rep = rgx.replace(&res, *new_val);
        match rep {
            std::borrow::Cow::Borrowed(_) => {}
            std::borrow::Cow::Owned(v) => {
                res = v;
            }
        }
    }

    let res = res.trim();
    format!("{}\n", res) //add final newline
}

fn has_minimum_address_components(addr: &Address) -> bool {
    // if there are neither 'road' nor 'postcode', we consider that there are not enough data
    // and use the fallback template
    addr[Component::Road].is_some() || addr[Component::Postcode].is_some()
}

impl ReplaceRule {
    fn replace_fields(&self, addr: &mut Address) {
        match self {
            ReplaceRule::All(replace_rule) => {
                for c in Component::iter() {
                    if let Some(v) = &addr[c] {
                        addr[c] = Some(
                            replace_rule
                                .regex
                                .replace(&v, replace_rule.replacement_value.as_str())
                                .to_string(),
                        );
                    }
                }
            }
            ReplaceRule::Component((c, replace_rule)) => {
                if let Some(v) = &addr[*c] {
                    addr[*c] = Some(
                        replace_rule
                            .regex
                            .replace(&v, replace_rule.replacement_value.as_str())
                            .to_string(),
                    );
                }
            }
        }
    }
}
