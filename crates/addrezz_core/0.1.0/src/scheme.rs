/// A URI scheme.
///
/// Known schemes have dedicated variants so that default ports and semantics
/// are available without string comparison. Unknown schemes fall back to
/// `Other`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum Scheme {
    /// HTTP
    Http,
    /// HTTPS
    Https,
    /// WebSocket
    Ws,
    /// WebSocket Secure
    Wss,
    /// Secure Shell
    Ssh,
    /// SSH File Transfer Protocol
    Sftp,
    /// Git
    Git,
    /// Git over SSH
    GitSsh,
    /// Git over HTTPS
    GitHttps,
    /// Git over HTTP
    GitHttp,
    /// Subversion
    Svn,
    /// Subversion over SSH
    SvnSsh,
    /// File Transfer Protocol
    Ftp,
    /// FTP over TLS
    Ftps,
    /// Local file access
    File,
    /// Inline data URI
    Data,
    /// Email address
    Mailto,
    /// Simple Mail Transfer Protocol
    Smtp,
    /// SMTP over TLS
    Smtps,
    /// Mail submission (port 587)
    Submission,
    /// Internet Message Access Protocol
    Imap,
    /// IMAP over TLS
    Imaps,
    /// Post Office Protocol v3
    Pop3,
    /// POP3 over TLS
    Pop3s,
    /// Lightweight Directory Access Protocol
    Ldap,
    /// LDAP over TLS
    Ldaps,
    /// PostgreSQL
    Postgres,
    /// MySQL
    Mysql,
    /// MariaDB
    Mariadb,
    /// MongoDB
    Mongodb,
    /// MongoDB SRV record
    MongodbSrv,
    /// Redis
    Redis,
    /// Redis over TLS
    Rediss,
    /// ClickHouse
    Clickhouse,
    /// Apache Cassandra
    Cassandra,
    /// SQLite
    Sqlite,
    /// Advanced Message Queuing Protocol
    Amqp,
    /// AMQP over TLS
    Amqps,
    /// MQTT
    Mqtt,
    /// MQTT over TLS
    Mqtts,
    /// NATS messaging
    Nats,
    /// Apache Kafka
    Kafka,
    /// gRPC
    Grpc,
    /// gRPC over TLS
    Grpcs,
    /// Session Initiation Protocol
    Sip,
    /// SIP over TLS
    Sips,
    /// Telephone number
    Tel,
    /// Extensible Messaging and Presence Protocol
    Xmpp,
    /// Internet Relay Chat
    Irc,
    /// IRC over TLS
    Ircs,
    /// Constrained Application Protocol
    Coap,
    /// CoAP over DTLS
    Coaps,
    /// STUN (Session Traversal Utilities for NAT)
    Stun,
    /// STUN over TLS
    Stuns,
    /// TURN (Traversal Using Relays around NAT)
    Turn,
    /// TURN over TLS
    Turns,
    /// Domain Name System
    Dns,
    /// Network Time Protocol
    Ntp,
    /// Unknown or unrecognized scheme
    Other(String),
}

impl Scheme {
    /// Parse a scheme from its lowercase ASCII form. Unknown schemes are
    /// returned as `Other`. Never fails; reject invalid schemes at the URI
    /// parser level instead.
    pub fn parse(s: &str) -> Self {
        let lower = s.to_ascii_lowercase();
        match lower.as_str() {
            "http" => Self::Http,
            "https" => Self::Https,
            "ws" => Self::Ws,
            "wss" => Self::Wss,
            "ssh" => Self::Ssh,
            "sftp" => Self::Sftp,
            "git" => Self::Git,
            "git+ssh" => Self::GitSsh,
            "git+https" => Self::GitHttps,
            "git+http" => Self::GitHttp,
            "svn" => Self::Svn,
            "svn+ssh" => Self::SvnSsh,
            "ftp" => Self::Ftp,
            "ftps" => Self::Ftps,
            "file" => Self::File,
            "data" => Self::Data,
            "mailto" => Self::Mailto,
            "smtp" => Self::Smtp,
            "smtps" => Self::Smtps,
            "submission" => Self::Submission,
            "imap" => Self::Imap,
            "imaps" => Self::Imaps,
            "pop3" => Self::Pop3,
            "pop3s" => Self::Pop3s,
            "ldap" => Self::Ldap,
            "ldaps" => Self::Ldaps,
            "postgres" | "postgresql" => Self::Postgres,
            "mysql" => Self::Mysql,
            "mariadb" => Self::Mariadb,
            "mongodb" => Self::Mongodb,
            "mongodb+srv" => Self::MongodbSrv,
            "redis" => Self::Redis,
            "rediss" => Self::Rediss,
            "clickhouse" => Self::Clickhouse,
            "cassandra" => Self::Cassandra,
            "sqlite" => Self::Sqlite,
            "amqp" => Self::Amqp,
            "amqps" => Self::Amqps,
            "mqtt" => Self::Mqtt,
            "mqtts" => Self::Mqtts,
            "nats" => Self::Nats,
            "kafka" => Self::Kafka,
            "grpc" => Self::Grpc,
            "grpcs" => Self::Grpcs,
            "sip" => Self::Sip,
            "sips" => Self::Sips,
            "tel" => Self::Tel,
            "xmpp" => Self::Xmpp,
            "irc" => Self::Irc,
            "ircs" => Self::Ircs,
            "coap" => Self::Coap,
            "coaps" => Self::Coaps,
            "stun" => Self::Stun,
            "stuns" => Self::Stuns,
            "turn" => Self::Turn,
            "turns" => Self::Turns,
            "dns" => Self::Dns,
            "ntp" => Self::Ntp,
            _ => Self::Other(lower),
        }
    }

    /// Return the canonical lowercase scheme string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Ws => "ws",
            Self::Wss => "wss",
            Self::Ssh => "ssh",
            Self::Sftp => "sftp",
            Self::Git => "git",
            Self::GitSsh => "git+ssh",
            Self::GitHttps => "git+https",
            Self::GitHttp => "git+http",
            Self::Svn => "svn",
            Self::SvnSsh => "svn+ssh",
            Self::Ftp => "ftp",
            Self::Ftps => "ftps",
            Self::File => "file",
            Self::Data => "data",
            Self::Mailto => "mailto",
            Self::Smtp => "smtp",
            Self::Smtps => "smtps",
            Self::Submission => "submission",
            Self::Imap => "imap",
            Self::Imaps => "imaps",
            Self::Pop3 => "pop3",
            Self::Pop3s => "pop3s",
            Self::Ldap => "ldap",
            Self::Ldaps => "ldaps",
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
            Self::Mariadb => "mariadb",
            Self::Mongodb => "mongodb",
            Self::MongodbSrv => "mongodb+srv",
            Self::Redis => "redis",
            Self::Rediss => "rediss",
            Self::Clickhouse => "clickhouse",
            Self::Cassandra => "cassandra",
            Self::Sqlite => "sqlite",
            Self::Amqp => "amqp",
            Self::Amqps => "amqps",
            Self::Mqtt => "mqtt",
            Self::Mqtts => "mqtts",
            Self::Nats => "nats",
            Self::Kafka => "kafka",
            Self::Grpc => "grpc",
            Self::Grpcs => "grpcs",
            Self::Sip => "sip",
            Self::Sips => "sips",
            Self::Tel => "tel",
            Self::Xmpp => "xmpp",
            Self::Irc => "irc",
            Self::Ircs => "ircs",
            Self::Coap => "coap",
            Self::Coaps => "coaps",
            Self::Stun => "stun",
            Self::Stuns => "stuns",
            Self::Turn => "turn",
            Self::Turns => "turns",
            Self::Dns => "dns",
            Self::Ntp => "ntp",
            Self::Other(s) => s.as_str(),
        }
    }

    /// IANA-registered default port, if any.
    pub fn default_port(&self) -> Option<u16> {
        match self {
            Self::Http | Self::Ws => Some(80),
            Self::Https | Self::Wss => Some(443),
            Self::Ssh | Self::Sftp | Self::GitSsh | Self::SvnSsh => Some(22),
            Self::Git => Some(9418),
            Self::GitHttps => Some(443),
            Self::GitHttp => Some(80),
            Self::Svn => Some(3690),
            Self::Ftp => Some(21),
            Self::Ftps => Some(990),
            Self::Smtp => Some(25),
            Self::Smtps => Some(465),
            Self::Submission => Some(587),
            Self::Imap => Some(143),
            Self::Imaps => Some(993),
            Self::Pop3 => Some(110),
            Self::Pop3s => Some(995),
            Self::Ldap => Some(389),
            Self::Ldaps => Some(636),
            Self::Postgres => Some(5432),
            Self::Mysql | Self::Mariadb => Some(3306),
            Self::Mongodb => Some(27017),
            Self::Redis | Self::Rediss => Some(6379),
            Self::Clickhouse => Some(9000),
            Self::Cassandra => Some(9042),
            Self::Amqp => Some(5672),
            Self::Amqps => Some(5671),
            Self::Mqtt => Some(1883),
            Self::Mqtts => Some(8883),
            Self::Nats => Some(4222),
            Self::Kafka => Some(9092),
            Self::Sip => Some(5060),
            Self::Sips => Some(5061),
            Self::Xmpp => Some(5222),
            Self::Irc => Some(6667),
            Self::Ircs => Some(6697),
            Self::Coap => Some(5683),
            Self::Coaps => Some(5684),
            Self::Stun | Self::Turn => Some(3478),
            Self::Stuns | Self::Turns => Some(5349),
            Self::Dns => Some(53),
            Self::Ntp => Some(123),
            // Schemes without a meaningful port.
            Self::File
            | Self::Data
            | Self::Mailto
            | Self::Tel
            | Self::Sqlite
            | Self::MongodbSrv
            | Self::Grpc
            | Self::Grpcs
            | Self::Other(_) => None,
        }
    }

    /// True if the scheme implies transport-level encryption.
    pub fn is_secure(&self) -> bool {
        matches!(
            self,
            Self::Https
                | Self::Wss
                | Self::Sftp
                | Self::Ftps
                | Self::GitHttps
                | Self::Smtps
                | Self::Submission
                | Self::Imaps
                | Self::Pop3s
                | Self::Ldaps
                | Self::Rediss
                | Self::Amqps
                | Self::Mqtts
                | Self::Grpcs
                | Self::Sips
                | Self::Ircs
                | Self::Coaps
                | Self::Stuns
                | Self::Turns
                | Self::MongodbSrv
        )
    }

    /// True if this is one of the SSH-family schemes.
    pub fn is_ssh_like(&self) -> bool {
        matches!(
            self,
            Self::Ssh | Self::Sftp | Self::GitSsh | Self::SvnSsh
        )
    }
}

impl core::fmt::Display for Scheme {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::str::FromStr for Scheme {
    type Err = core::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}
