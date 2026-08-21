use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataWrapper<T> {
    pub data: T,
}

// serde chrono DateTime to iso8601 string
pub(crate) fn serde_datetime_to_string<S>(
    datetime: &chrono::DateTime<chrono::Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&datetime.to_rfc3339())
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[repr(u8)]
pub enum Category {
    /// Altering DNS records resulting in improper redirection.
    DnsCompromise = 1,
    /// Falsifying domain server cache (cache poisoning).
    DnsPoisoning = 2,
    /// Fraudulent orders.
    FraudOrder = 3,
    /// Participating in distributed denial-of-service (usually part of botnet).
    DdosAttack = 4,
    /// FTP Brute-Force
    FtpBruteForce = 5,
    /// Oversized IP packet.
    PingOfDeath = 6,
    /// Phishing websites and/or email.
    Phishing = 7,
    /// Fraud VoIP
    FraudVoip = 8,
    /// Open proxy, open relay, or Tor exit node.
    OpenProxy = 9,
    /// Comment/forum spam, HTTP referer spam, or other CMS spam.
    WebSpam = 10,
    /// Spam email content, infected attachments, and phishing emails.
    /// Note: Limit comments to only relevent information (instead of log dumps)
    /// and be sure to remove PII if you want to remain anonymous.
    EmailSpam = 11,
    /// CMS blog comment spam.
    BlogSpam = 12,
    /// VPN IP - Conjunctive category.
    VpnIp = 13,
    /// Scanning for open ports and vulnerable services.
    PortScan = 14,
    /// Hacking
    Hacking = 15,
    /// Attempts at SQL injection.
    SqlInjection = 16,
    /// Email sender spoofing.
    Spoofing = 17,
    /// Brute-force attacks on webpage logins and services
    /// like SSH, FTP, SIP, SMTP, RDP, etc.
    /// This category is seperate from DDoS attacks.
    BruteForceCredential = 18,
    /// Webpage scraping (for email addresses, content, etc) and crawlers that
    /// do not honor robots.txt. Excessive requests and user agent spoofing
    /// can also be reported here.
    BadWebBot = 19,
    /// Host is likely infected with malware and being used for other
    /// attacks or to host malicious content. The host owner may not be aware
    /// of the compromise. This category is often used in combination with
    /// other attack categories.
    ExploitedHost = 20,
    /// Attempts to probe for or exploit installed web applications such
    /// as a CMS like WordPress/Drupal, e-commerce solutions, forum software,
    /// phpMyAdmin and various other software plugins/solutions.
    WebAppAttack = 21,
    /// Secure Shell (SSH) abuse. Use this category in combination with more
    /// specific categories.
    SshAbuse = 22,
    /// Abuse was targeted at an "Internet of Things" type device.
    /// Include information about what type of device was targeted
    /// in the comments.
    IotTargeted = 23,
}

// serde serialize vec of categories to comma separated string, by using the repr of the enum
pub(crate) fn serde_categories_to_string<S>(
    categories: &[Category],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut s = String::new();
    for category in categories {
        if !s.is_empty() {
            s.push(',');
        }
        s.push_str(&(category.clone() as u8).to_string());
    }
    serializer.serialize_str(&s)
}

pub(crate) fn serde_vec_to_categories<'de, D>(deserializer: D) -> Result<Vec<Category>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Vec::deserialize(deserializer)?;
    let mut categories = Vec::new();
    for part in s {
        match part {
            1 => categories.push(Category::DnsCompromise),
            2 => categories.push(Category::DnsPoisoning),
            3 => categories.push(Category::FraudOrder),
            4 => categories.push(Category::DdosAttack),
            5 => categories.push(Category::FtpBruteForce),
            6 => categories.push(Category::PingOfDeath),
            7 => categories.push(Category::Phishing),
            8 => categories.push(Category::FraudVoip),
            9 => categories.push(Category::OpenProxy),
            10 => categories.push(Category::WebSpam),
            11 => categories.push(Category::EmailSpam),
            12 => categories.push(Category::BlogSpam),
            13 => categories.push(Category::VpnIp),
            14 => categories.push(Category::PortScan),
            15 => categories.push(Category::Hacking),
            16 => categories.push(Category::SqlInjection),
            17 => categories.push(Category::Spoofing),
            18 => categories.push(Category::BruteForceCredential),
            19 => categories.push(Category::BadWebBot),
            20 => categories.push(Category::ExploitedHost),
            21 => categories.push(Category::WebAppAttack),
            22 => categories.push(Category::SshAbuse),
            23 => categories.push(Category::IotTargeted),
            _ => return Err(serde::de::Error::custom("invalid category")),
        }
    }

    Ok(categories)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Error {
    #[serde(rename = "detail")]
    message: String,
    status: u32,
}

pub(crate) fn serde_duration_to_days<S>(
    duration: &Option<Duration>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match duration {
        Some(duration) => serializer.serialize_some(&duration.num_days()),
        None => serializer.serialize_none(),
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// TODO: This is a string in the API, but it's actually a date
    reported_at: String,
    comment: String,
    #[serde(deserialize_with = "serde_vec_to_categories")]
    categories: Vec<Category>,
    reporter_id: u32,
    reporter_country_code: String,
    reporter_country_name: String,
}

#[derive(Debug, Clone)]
pub enum IpVersion {
    V4,
    V6,
}

pub(crate) fn serde_ip_version_from_i32<'de, D>(deserializer: D) -> Result<IpVersion, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = i32::deserialize(deserializer)?;
    match s {
        4 => Ok(IpVersion::V4),
        6 => Ok(IpVersion::V6),
        _ => Err(serde::de::Error::custom("invalid IP version")),
    }
}

pub(crate) fn serde_ip_version_to_i32<S>(
    version: &IpVersion,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match version {
        IpVersion::V4 => serializer.serialize_i32(4),
        IpVersion::V6 => serializer.serialize_i32(6),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum UsageType {
    Commercial,
    Organization,
    Government,
    Military,
    University,
    Library,
    ContentDeliveryNetwork,
    FixedLineISP,
    MobileISP,
    DataCenter,
    SearchEngineSpider,
    Reserved,
}

pub(crate) fn serde_usage_type_from_str<'de, D>(deserializer: D) -> Result<Vec<UsageType>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mut types = Vec::new();
    for part in s.split('/') {
        match part {
            "Commercial" => types.push(UsageType::Commercial),
            "Organization" => types.push(UsageType::Organization),
            "Government" => types.push(UsageType::Government),
            "Military" => types.push(UsageType::Military),
            "University" => types.push(UsageType::University),
            "Library" => types.push(UsageType::Library),
            "Content Delivery Network" => types.push(UsageType::ContentDeliveryNetwork),
            "Fixed Line ISP" => types.push(UsageType::FixedLineISP),
            "Mobile ISP" => types.push(UsageType::MobileISP),
            "Data Center" => types.push(UsageType::DataCenter),
            "Search Engine Spider" => types.push(UsageType::SearchEngineSpider),
            "Reserved" => types.push(UsageType::Reserved),
            _ => return Err(serde::de::Error::custom("invalid usage type")),
        }
    }
    Ok(types)
}
