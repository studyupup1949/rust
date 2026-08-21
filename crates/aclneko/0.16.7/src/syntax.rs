use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

const IPV4_ADDR: &str = r"((\d{1,3}\.){3}\d{1,3}(-(\d{1,3}\.){3}\d{1,3})?)";
const IPV6_ADDR: &str = r"((([[:xdigit:]]{1,4}){1,4}::([[:xdigit:]]{1,4})?|(([[:xdigit:]]{1,4}):){5}([[:xdigit:]]{1,4}))(-(([[:xdigit:]]{1,4}){1,4}::([[:xdigit:]]{1,4})?|(([[:xdigit:]]{1,4}):){5}([[:xdigit:]]{1,4})))?)";

/// Category represents line category for policy syntax.
/// Any policy line should be one of the category below.
/// If unrecognizable category found, an application
/// may generate error message or panic for users.
///
#[derive(Deserialize, Serialize, PartialEq, Eq)]
pub enum Category {
    Version,
    Stat,
    Quota,
    NumberGroupDef,
    IpGroupDef,
    StringGroupDef,
    Header,
    Audit,
    Rule,
    Blank,
    Comment,
    Error,
}

/// Matcher holds syntax pattern for lexical/syntax analisys with
/// regex::Regex.
///
pub struct Matcher {
    version: Regex,
    stat: Regex,
    quota_memory: Regex,
    quota_audit: Regex,
    number_group: Regex,
    ip_group: Regex,
    string_group: Regex,
    acl_header: Regex,
    acl_audit: Regex,
    acl_rule: Regex,
    attr: Regex,
    blank: Regex,
    comment: Regex,
}

/// Matcher holds preset regex in its private fields and it is instanciated before a
/// parsing. Any policy syntax for c6h is parsed with methods in implemented here.
///
impl Matcher {
    pub fn new() -> Self {
        // generic pattern
        let init = String::from(r"^");
        let priority = r"(?P<priority>[[:digit:]]+)";

        // literals
        let number_literal = r"([[:digit:]]+(-[[:digit:]]+)?)";
        let ip_literal = r"((\d{1,3}\.){3}\d{1,3}(-(\d{1,3}\.){3}\d{1,3})?|(([[:xdigit:]]{4})?((:[[:xdigit:]]{4})*:(:?[[:xdigit:]]{1,4}|:))))";
        let key = r"[[:alpha:]][-_[:alnum:]]*";
        let group = String::from("@") + &key;
        let string_literal = r"[-\./\\\*\(\)\+><[:alpha:]][-\.~@\?:\(\*\[\]\$\+\\\)></_[:alnum:]]*";

        // group-related patterns
        let operation = String::from("(?P<operation>") + Op::list_str().join("|").as_str() + ")";
        let number_group_pattern =
            init.clone() + r"number_group\s+(" + &key + r")\s+" + number_literal;
        let number_group_pattern = number_group_pattern.as_str();
        let ip_group_pattern = init.clone() + r"ip_group\s+(" + &key + r")\s+" + ip_literal;
        let string_group_pattern =
            init.clone() + r"string_group\s+(" + &key + r")\s+(" + &string_literal + r")";

        // attr-related patterns

        // A resurce is the key of an attribute, which is in the set of registered words
        let resource =
            String::from(r"(?P<resource>") + Resource::list_str().join("|").as_str() + ")";
        // A conditional operator is the operation for an attributes, which is = or !=
        let conditional_op = r"(?P<condition>!?=)";
        // A attribute target is the value of an attributes which is number, string or any type of group following to group operator `@`
        let attr_target = String::from(r"(?P<target>")
            + "("
            + IPV4_ADDR
            + "|"
            + IPV6_ADDR
            + "|"
            + r"\d+(-\d+)?"
            + "|"
            + "(" //r1
            + Resource::list_str().join("|").as_str()
            + ")" //l1
            + "|"
            + r"\x22"
            + &string_literal
            + r"\x22"
            + "|"
            + &group
            + "))";
        let attr_target = attr_target.as_str();
        // An attribute is a pair of resource, conditional operator and target.
        let attribute_pattern = resource + conditional_op + &attr_target;

        let attr_list =
            String::from(r"(?P<attrs>(\s+") + attribute_pattern.clone().as_str() + ")*)";
        let attr_list = attr_list.as_str();

        // ACL header, ACL audit and ACL rule patterns
        let e = r"\s*$";
        // header
        let acl_header_pattern =
            init.clone() + &priority + r"\s+acl\s+" + &operation.as_str() + &attr_list + &e;
        // audit
        let acl_audit_pattern = init.clone() + r"\s+(?P<verb>audit)\s+" + r"(?P<seq>\d+)" + &e;
        // rule
        let acl_rule_pattern =
            init.clone() + r"\s+" + &priority + r"\s+(?P<verb>allow|deny)" + &attr_list + &e;

        // println!("attr_pat: {}", attr_list);
        Matcher {
            version: Regex::new(r"^POLICY_VERSION=(\d+)").unwrap(),
            stat: Regex::new(r"^stat [^\s]+\s+[^\s]+").unwrap(),
            quota_memory: Regex::new(r"^quota\s+memory\s+(policy|audit|query)\s+\d+").unwrap(),
            quota_audit: Regex::new(
                r"^quota\s+audit\[\d+\]\s+allowed=\d+\s+denied=\d+\s+unmatched=\d+",
            )
            .unwrap(),
            number_group: Regex::new(number_group_pattern).unwrap(),
            ip_group: Regex::new(&ip_group_pattern).unwrap(),
            string_group: Regex::new(&string_group_pattern).unwrap(),
            acl_header: Regex::new(&acl_header_pattern).unwrap(),
            acl_audit: Regex::new(&acl_audit_pattern).unwrap(),
            acl_rule: Regex::new(&acl_rule_pattern).unwrap(),
            attr: Regex::new(&attribute_pattern).unwrap(),
            blank: Regex::new(r"^\s*$").unwrap(),
            comment: Regex::new(r"^\s*#.*$").unwrap(),
        }
    }

    ///gc
    ///
    ///```rust
    ///use aclneko::syntax::*;
    ///
    ///let m = Matcher::new();
    ///let c = m.parse_category("1000 acl read");
    ///assert_eq!(c == Category::Header, true);
    ///```
    ///
    pub fn parse_category(&self, line: &str) -> Category {
        if self.is_version(line) {
            Category::Version
        } else if self.is_stat(line) {
            Category::Stat
        } else if self.is_memory_quota(line) {
            Category::Quota
        } else if self.is_audit_quota(line) {
            Category::Quota
        } else if self.is_number_group(line) {
            Category::NumberGroupDef
        } else if self.is_ip_group(line) {
            Category::IpGroupDef
        } else if self.is_string_group(line) {
            Category::StringGroupDef
        } else if self.is_acl_header(line) {
            Category::Header
        } else if self.is_acl_audit(line) {
            Category::Audit
        } else if self.is_acl_rule(line) {
            Category::Rule
        } else if self.is_blank(line) {
            Category::Blank
        } else if self.is_comment(line) {
            Category::Comment
        } else {
            Category::Error
        }
    }

    pub fn is_version(&self, line: &str) -> bool {
        self.version.is_match(line)
    }

    pub fn parse_version(&self, line: &str) -> u32 {
        self.version.captures(&line).unwrap()[1]
            .to_string()
            .parse::<u32>()
            .unwrap()
    }

    pub fn is_stat(&self, line: &str) -> bool {
        self.stat.is_match(line)
    }

    pub fn is_memory_quota(&self, line: &str) -> bool {
        self.quota_memory.is_match(line)
    }

    pub fn is_audit_quota(&self, line: &str) -> bool {
        self.quota_audit.is_match(line)
    }

    pub fn is_number_group(&self, line: &str) -> bool {
        self.number_group.is_match(line)
    }

    pub fn parse_number_group(&self, line: &str) -> (String, String) {
        let mat = self.number_group.captures(&line).unwrap();
        (mat[1].to_string(), mat[2].to_string())
    }

    pub fn is_ip_group(&self, line: &str) -> bool {
        self.ip_group.is_match(line)
    }

    pub fn parse_ip_group(&self, line: &str) -> (String, String) {
        let mat = self.ip_group.captures(&line).unwrap();
        (mat[1].to_string(), mat[2].to_string())
    }

    pub fn is_string_group(&self, line: &str) -> bool {
        self.string_group.is_match(line)
    }

    pub fn parse_string_group(&self, line: &str) -> (String, String) {
        let mat = self.string_group.captures(&line).unwrap();
        (mat[1].to_string(), mat[2].to_string())
    }

    pub fn is_acl_header(&self, line: &str) -> bool {
        self.acl_header.is_match(line)
    }

    /// parse_acl_header returns acl header properties within a tuple.
    /// Returned tuple holds acl priority, Operation and optional attribute for
    /// the Operation.
    ///
    pub fn parse_acl_header(&self, line: &str) -> (u16, Op, Vec<(Resource, Cond, String)>) {
        let cap = &self.acl_header.captures(line).unwrap();
        (
            cap["priority"].to_string().parse::<u16>().unwrap(),
            Op::from(&cap["operation"]),
            self.parse_attr_list(&cap["attrs"].to_string()),
        )
    }

    /// parse_attr_list returns a list of attributes which conposes optional ACL modifier.
    ///
    pub fn parse_attr_list(&self, line: &str) -> Vec<(Resource, Cond, String)> {
        let mut res: Vec<(Resource, Cond, String)> = Vec::new();
        for cap in self.attr.captures_iter(line) {
            res.push((
                Resource::from(&cap["resource"]),
                Cond::from(&cap["condition"]),
                cap["target"].to_string(),
            ));
        }
        res
    }

    /// is_acl_audit detects whether the line is a valid
    /// ACL audit line or not.
    pub fn is_acl_audit(&self, line: &str) -> bool {
        self.acl_audit.is_match(line)
    }

    /// parse_acl_audit_sequence returns audit sequence value
    /// set to an audit line.
    /// Any lines supplied to this function must be a valid
    /// audit lines which is parsable with self.acl_audit matcher.
    ///
    pub fn parse_acl_audit_sequence(&self, line: &str) -> u16 {
        let cap = &self.acl_audit.captures(line).unwrap();
        cap["seq"].to_string().parse::<u16>().unwrap()
    }

    pub fn is_blank(&self, line: &str) -> bool {
        self.blank.is_match(line)
    }

    pub fn is_comment(&self, line: &str) -> bool {
        self.comment.is_match(line)
    }

    /// is_acl_rule detects whether the line is a valid
    /// ACL rule line or not.
    pub fn is_acl_rule(&self, line: &str) -> bool {
        self.acl_rule.is_match(line)
    }

    /// parse_acl_rule returns ACL rule properties within a tuple,
    /// which contains rule priority, verb (allow/deny) and attribute
    /// list in vector format.
    /// Any lines supplied to this function must be a valid rule line
    /// which is ensured to be parsed with self.acl_rule matcher.
    ///
    pub fn parse_acl_rule(&self, line: &str) -> (u16, Verb, Vec<(Resource, Cond, String)>) {
        let cap = &self.acl_rule.captures(line).unwrap();
        (
            cap["priority"].to_string().parse::<u16>().unwrap(),
            Verb::from(&cap["verb"]),
            self.parse_attr_list(&cap["attrs"].to_string()),
        )
    }
}

pub struct AuditMatcher {
    header: Regex,
    body: Regex,
    operation: Regex,
    uid: Regex,
    ip: Regex,
    port: Regex,
    task: Regex,
    task_exe: Regex,
    task_domain: Regex,
    task_ugid: Regex,
    task_xpid: Regex,
    path: Regex,
    path_single: Regex,
    path_ugid: Regex,
    path_mod: Regex,
    path_parent: Regex,
    path_parent_mod: Regex,
    path_parent_ugid: Regex,
    transition: Regex,
}

impl AuditMatcher {
    pub fn new() -> AuditMatcher {
        let header = r"^#(?P<timestamp>[^#]+)# global-pid=(?P<gpid>\d+) result=(allow|deny|unmatched) priority=(?P<priority>\d+) / ";
        AuditMatcher {
            header: Regex::new(header).unwrap(),
            body: Regex::new(format!(r"{}(?P<body>.+)", header).as_str()).unwrap(),
            operation: Regex::new(format!(r"[\s^]({})[$\s]", Op::list_str().join("|")).as_str())
                .unwrap(),
            uid: Regex::new(r"[\s^]uid=\d+[\s$]").unwrap(),
            ip: Regex::new(
                format!(
                    "ip={}",
                    String::from("(") + IPV4_ADDR + "|" + IPV6_ADDR + ")"
                )
                .as_str(),
            )
            .unwrap(),
            port: Regex::new(r"port=\d+").unwrap(),
            task: Regex::new(r"task(\.[^\s]+)+=[^\s]+").unwrap(),
            task_exe: Regex::new(r"task\.exe=[^\s]+").unwrap(),
            task_ugid: Regex::new(r"task\.[ug]id=\d+").unwrap(),
            task_xpid: Regex::new(r"task\.[p]?pid=\d+").unwrap(),
            task_domain: Regex::new(r"task\.domain=[^\s]+").unwrap(),
            path: Regex::new(r"path(\.[^\s\.]+)?=[^\s]+").unwrap(),
            path_single: Regex::new(r"path=[^\s]+").unwrap(),
            path_ugid: Regex::new(r"path\.[ug]id=\d+").unwrap(),
            path_mod: Regex::new(r"path\.perm=\d+").unwrap(),
            path_parent: Regex::new(r"path\.parent(\.[^\s]+)?=[^\s]+").unwrap(),
            path_parent_mod: Regex::new(r"path\.parent\.perm=\d+").unwrap(),
            path_parent_ugid: Regex::new(r"path\.parent\.[ug]id=\d+").unwrap(),
            transition: Regex::new(r"transition=[^\s]+").unwrap(),
        }
    }

    pub fn has_header(&self, line: &str) -> bool {
        self.header.is_match(line)
    }
    pub fn parse_header(&self, line: &str) -> Option<String> {
        match self.header.captures(&line) {
            Some(cap) => Some(String::from(&cap[0])),
            None => None,
        }
    }
    pub fn parse_body(&self, line: &str) -> Option<String> {
        match self.body.captures(&line) {
            Some(cap) => Some(String::from(&cap["body"])),
            None => None,
        }
    }
    pub fn is_operation(&self, line: &str) -> bool {
        self.operation.is_match(line)
    }
    pub fn parse_operation(&self, line: &str) -> Option<String> {
        match self.operation.captures(&line) {
            Some(cap) => Some(String::from(&cap[0])),
            None => None,
        }
    }
    pub fn is_uid(&self, line: &str) -> bool {
        self.uid.is_match(line)
    }
    pub fn is_ip(&self, line: &str) -> bool {
        self.ip.is_match(line)
    }
    pub fn is_port(&self, line: &str) -> bool {
        self.port.is_match(line)
    }
    pub fn is_task(&self, line: &str) -> bool {
        self.task.is_match(line)
    }
    pub fn is_task_exe(&self, line: &str) -> bool {
        self.task_exe.is_match(line)
    }
    pub fn is_task_ugid(&self, line: &str) -> bool {
        self.task_ugid.is_match(line)
    }
    pub fn is_task_xpid(&self, line: &str) -> bool {
        self.task_xpid.is_match(line)
    }
    pub fn is_task_domain(&self, line: &str) -> bool {
        self.task_domain.is_match(line)
    }
    pub fn is_path(&self, line: &str) -> bool {
        self.path.is_match(line)
    }
    pub fn is_path_single(&self, line: &str) -> bool {
        self.path_single.is_match(line)
    }
    pub fn is_path_ugid(&self, line: &str) -> bool {
        self.path_ugid.is_match(line)
    }
    pub fn is_path_mod(&self, line: &str) -> bool {
        self.path_mod.is_match(line)
    }
    pub fn is_path_parent(&self, line: &str) -> bool {
        self.path_parent.is_match(line)
    }
    pub fn is_path_parent_ugid(&self, line: &str) -> bool {
        self.path_parent_ugid.is_match(line)
    }
    pub fn is_path_parent_mod(&self, line: &str) -> bool {
        self.path_parent_mod.is_match(line)
    }
    pub fn is_transition(&self, line: &str) -> bool {
        self.transition.is_match(line)
    }
}

/// Verb is c6h's verb on rule lines.
/// A verb must be exist on the head of a rule line except for `audit`,
/// which is the second verb following to the audit priority digit.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Verb {
    Allow,
    Deny,
    Audit,
    Error,
}

impl Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for Verb {
    fn from(res: &str) -> Self {
        match res {
            "allow" => Self::Allow,
            "deny" => Self::Deny,
            "audit" => Self::Audit,
            _ => Self::Error,
        }
    }
}

impl Verb {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Audit => "audit",
            Self::Error => "",
        }
    }
}

/// Op represents system opration which is recognized by c6h.
///
#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Op {
    Read,
    Write,
    Create,
    Execute,
    Append,
    Truncate,
    Mkdir,
    Rmdir,
    Unlink,
    Link,
    Symlink,
    Rename,
    Chmod,
    Chown,
    Chgrp,
    Mount,
    Unmount,
    Chroot,
    Mkfifo,
    Mksock,
    Mkblock,
    Mkchar,
    Ioctl,
    PivotRoot,
    UnixDgramBind,
    UnixDgramSend,
    UnixDgramRecv,
    UnixStreamBind,
    UnixStreamListen,
    UnixStreamConnect,
    UnixStreamAccept,
    UnixSeqpacketBind,
    UnixSeqpacketListen,
    UnixSeqpacketConnect,
    UnixSeqpacketAccept,
    InetDgramBind,
    InetDgramSend,
    InetDgramRecv,
    InetRawBind,
    InetRawSend,
    InetRawRecv,
    InetStreamBind,
    InetStreamListen,
    InetStreamConnect,
    InetStreamAccept,
    Ptrace,
    Signal,
    Environ,
    ModifyPolicy,
    UseNetlinkSocket,
    UsePacketSocket,
    UseReboot,
    UseVhangup,
    SetTime,
    SetPriority,
    SetHostname,
    UseKernelModule,
    UseNewKernel,
    ManualDomainTransition,
    AutoDomainTransition,
    Error,
}

impl Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for Op {
    fn from(op: &str) -> Self {
        let res = match op.trim() {
            "read" => Self::Read,
            "write" => Self::Write,
            "create" => Self::Create,
            "execute" => Self::Execute,
            "append" => Self::Append,
            "truncate" => Self::Truncate,
            "mkdir" => Self::Mkdir,
            "rmdir" => Self::Rmdir,
            "unlink" => Self::Unlink,
            "link" => Self::Link,
            "symlink" => Self::Symlink,
            "rename" => Self::Rename,
            "chmod" => Self::Chmod,
            "chown" => Self::Chown,
            "chgrp" => Self::Chgrp,
            "mount" => Self::Mount,
            "unmount" => Self::Unmount,
            "chroot" => Self::Chroot,
            "mkfifo" => Self::Mkfifo,
            "mksock" => Self::Mksock,
            "mkblock" => Self::Mkblock,
            "mkchar" => Self::Mkchar,
            "ioctl" => Self::Ioctl,
            "pivot_root" => Self::PivotRoot,
            "unix_dgram_bind" => Self::UnixDgramBind,
            "unix_dgram_send" => Self::UnixDgramSend,
            "unix_dgram_recv" => Self::UnixDgramRecv,
            "unix_stream_bind" => Self::UnixStreamBind,
            "unix_stream_listen" => Self::UnixStreamListen,
            "unix_stream_connect" => Self::UnixStreamConnect,
            "unix_stream_accept" => Self::UnixStreamAccept,
            "unix_seqpacket_bind" => Self::UnixSeqpacketBind,
            "unix_seqpacket_listen" => Self::UnixSeqpacketListen,
            "unix_seqpacket_connect" => Self::UnixSeqpacketConnect,
            "unix_seqpacket_accept" => Self::UnixSeqpacketAccept,
            "inet_dgram_bind" => Self::InetDgramBind,
            "inet_dgram_send" => Self::InetDgramSend,
            "inet_dgram_recv" => Self::InetDgramRecv,
            "inet_raw_bind" => Self::InetRawBind,
            "inet_raw_send" => Self::InetRawSend,
            "inet_raw_recv" => Self::InetRawRecv,
            "inet_stream_bind" => Self::InetStreamBind,
            "inet_stream_listen" => Self::InetStreamListen,
            "inet_stream_connect" => Self::InetStreamConnect,
            "inet_stream_accept" => Self::InetStreamAccept,
            "ptrace" => Self::Ptrace,
            "signal" => Self::Signal,
            "environ" => Self::Environ,
            "modify_policy" => Self::ModifyPolicy,
            "use_netlink_socket" => Self::UseNetlinkSocket,
            "use_packet_socket" => Self::UsePacketSocket,
            "use_reboot" => Self::UseReboot,
            "use_vhangup" => Self::UseVhangup,
            "set_time" => Self::SetTime,
            "set_priority" => Self::SetPriority,
            "set_hostname" => Self::SetHostname,
            "use_kernel_module" => Self::UseKernelModule,
            "use_new_kernel" => Self::UseNewKernel,
            "manual_domain_transition" => Self::ManualDomainTransition,
            "auto_domain_transition" => Self::AutoDomainTransition,
            _ => Self::Error,
        };
        res
    }
}

impl Op {
    pub fn list() -> Vec<Op> {
        vec![
            Op::Read,
            Op::Write,
            Op::Create,
            Op::Execute,
            Op::Append,
            Op::Truncate,
            Op::Mkdir,
            Op::Rmdir,
            Op::Unlink,
            Op::Link,
            Op::Symlink,
            Op::Rename,
            Op::Chmod,
            Op::Chown,
            Op::Chgrp,
            Op::Ioctl,
            Op::Mount,
            Op::Unmount,
            Op::Chroot,
            Op::Mksock,
            Op::Mkblock,
            Op::Mkchar,
            Op::PivotRoot,
            Op::UnixDgramBind,
            Op::UnixDgramSend,
            Op::UnixDgramRecv,
            Op::UnixStreamBind,
            Op::UnixStreamListen,
            Op::UnixStreamConnect,
            Op::UnixStreamAccept,
            Op::UnixSeqpacketBind,
            Op::UnixSeqpacketListen,
            Op::UnixSeqpacketConnect,
            Op::UnixSeqpacketAccept,
            Op::InetDgramBind,
            Op::InetDgramSend,
            Op::InetDgramRecv,
            Op::InetRawBind,
            Op::InetRawSend,
            Op::InetRawRecv,
            Op::InetStreamBind,
            Op::InetStreamListen,
            Op::InetStreamConnect,
            Op::InetStreamAccept,
            Op::Ptrace,
            Op::Signal,
            Op::Environ,
            Op::ModifyPolicy,
            Op::UseNetlinkSocket,
            Op::UsePacketSocket,
            Op::UseReboot,
            Op::UseVhangup,
            Op::SetTime,
            Op::SetPriority,
            Op::SetHostname,
            Op::UseKernelModule,
            Op::UseNewKernel,
            Op::ManualDomainTransition,
            Op::AutoDomainTransition,
        ]
    }

    /// detect the given string and returns a list of operations which cooresponds
    /// to the given expression. A valid operation string is parsed as a normal
    /// and returns single operation in a list. Any of special operations,
    /// (all, net, modify or access), returns a set of operations.
    ///
    pub fn detect(op: &str) -> Result<Vec<Op>, String> {
        Ok(match op {
            "all" => Op::list(),
            "net" => vec![Op::InetDgramSend, Op::InetStreamConnect],
            "modify" => vec![
                Op::Write,
                Op::Create,
                Op::Append,
                Op::Truncate,
                Op::Mkdir,
                Op::Rmdir,
                Op::Unlink,
                Op::Link,
                Op::Rename,
                Op::Chmod,
                Op::Chown,
                Op::Chgrp,
            ],
            "access" => vec![
                Op::Read,
                Op::Write,
                Op::Execute,
                Op::Create,
                Op::Append,
                Op::Truncate,
                Op::Mkdir,
                Op::Rmdir,
                Op::Unlink,
                Op::Link,
                Op::Rename,
                Op::Chmod,
                Op::Chown,
                Op::Chgrp,
            ],
            _ => {
                let op = Op::from(op);
                if op == Op::Error {
                    return Err(format!("no such operation: {}", op));
                }
                vec![op]
            }
        })
    }
    pub fn is_access(&self) -> bool {
        match self {
            Self::Read
            | Self::Write
            | Self::Execute
            | Self::Create
            | Self::Append
            | Self::Truncate
            | Self::Mkdir
            | Self::Rmdir
            | Self::Unlink
            | Self::Link
            | Self::Rename
            | Self::Chmod
            | Self::Chown
            | Self::Chgrp => true,
            _ => false,
        }
    }
    pub fn is_modification(&self) -> bool {
        match self {
            Self::Write
            | Self::Create
            | Self::Append
            | Self::Truncate
            | Self::Mkdir
            | Self::Rmdir
            | Self::Unlink
            | Self::Link
            | Self::Rename
            | Self::Chmod
            | Self::Chown
            | Self::Chgrp => true,
            _ => false,
        }
    }
    pub fn is_net(&self) -> bool {
        match self {
            Self::InetDgramBind
            | Self::InetDgramSend
            | Self::InetDgramRecv
            | Self::InetStreamBind
            | Self::InetStreamListen
            | Self::InetStreamConnect
            | Self::InetStreamAccept
            | Self::InetRawBind
            | Self::InetRawSend
            | Self::InetRawRecv => true,
            _ => false,
        }
    }

    pub fn is_kernel(&self) -> bool {
        match self {
            Self::UseNewKernel | Self::UseKernelModule => true,
            _ => false,
        }
    }

    pub fn is_system_modification(&self) -> bool {
        match self {
            Self::UseKernelModule
            | Self::UseNewKernel
            | Self::SetTime
            | Self::SetPriority
            | Self::SetHostname => true,
            _ => false,
        }
    }

    pub fn is_power_management(&self) -> bool {
        match self {
            Self::UseReboot | Self::UseVhangup => true,
            _ => false,
        }
    }

    pub fn list_str() -> Vec<&'static str> {
        let mut res = vec![];
        for o in Op::list() {
            res.push(o.as_str());
        }
        res
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Create => "create",
            Self::Execute => "execute",
            Self::Append => "append",
            Self::Truncate => "truncate",
            Self::Mkdir => "mkdir",
            Self::Rmdir => "rmdir",
            Self::Unlink => "unlink",
            Self::Link => "link",
            Self::Symlink => "symlink",
            Self::Rename => "rename",
            Self::Chmod => "chmod",
            Self::Chown => "chown",
            Self::Chgrp => "chgrp",
            Self::Ioctl => "ioctl",
            Self::Mount => "mount",
            Self::Unmount => "unmount",
            Self::Chroot => "chroot",
            Self::Mksock => "mksock",
            Self::Mkfifo => "mkfifo",
            Self::Mkblock => "mkblok",
            Self::Mkchar => "mkchar",
            Self::PivotRoot => "pivot_root",
            Self::UnixDgramBind => "unix_dgram_bind",
            Self::UnixDgramSend => "unix_dgram_send",
            Self::UnixDgramRecv => "unix_dgram_recv",
            Self::UnixStreamBind => "unix_stream_bind",
            Self::UnixStreamListen => "unix_stream_listen",
            Self::UnixStreamConnect => "unix_stream_connect",
            Self::UnixStreamAccept => "unix_stream_accept",
            Self::UnixSeqpacketBind => "unix_seqpacket_bind",
            Self::UnixSeqpacketListen => "unix_seqpacket_listen",
            Self::UnixSeqpacketConnect => "unix_seqpacket_connect",
            Self::UnixSeqpacketAccept => "unix_seqpacket_accept",
            Self::InetDgramBind => "inet_dgram_bind",
            Self::InetDgramSend => "inet_dgram_send",
            Self::InetDgramRecv => "inet_dgram_recv",
            Self::InetRawBind => "inet_raw_bind",
            Self::InetRawSend => "inet_raw_send",
            Self::InetRawRecv => "inet_raw_recv",
            Self::InetStreamBind => "inet_stream_bind",
            Self::InetStreamListen => "inet_stream_listen",
            Self::InetStreamConnect => "inet_stream_connect",
            Self::InetStreamAccept => "inet_stream_accept",
            Self::Ptrace => "ptrace",
            Self::Signal => "signal",
            Self::Environ => "environ",
            Self::ModifyPolicy => "modify_policy",
            Self::UseNetlinkSocket => "use_netlink_socket",
            Self::UsePacketSocket => "use_packet_socket",
            Self::UseReboot => "use_reboot",
            Self::UseVhangup => "use_vhangup",
            Self::SetTime => "set_time",
            Self::SetPriority => "set_priority",
            Self::SetHostname => "set_hostname",
            Self::UseKernelModule => "use_kernel_module",
            Self::UseNewKernel => "use_new_kernel",
            Self::ManualDomainTransition => "manual_domain_transition",
            Self::AutoDomainTransition => "auto_domain_transition",
            Self::Error => "",
        }
    }
}

/// Resouce reprensents various resources recognized by c6h.
///
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Resource {
    TaskPid,
    TaskPpid,
    TaskUid,
    TaskEuid,
    TaskEgid,
    TaskSuid,
    TaskSgid,
    TaskFsuid,
    TaskFsgid,
    TaskExe,
    TaskDomain,
    TaskType,
    Perm,
    DevMajor,
    DevMinor,
    Path,
    PathType,
    PathIno,
    PathPerm,
    PathUid,
    PathGid,
    PathMajor,
    PathMinor,
    PathDevMajor,
    PathDevMinor,
    PathFsmagic,
    PathParent,
    PathParentIno,
    PathParentType,
    PathParentPerm,
    PathParentUid,
    PathParentGid,
    PathParentMajor,
    PathParentMinor,
    PathParentDevMajor,
    PathParentDevMinor,
    PathParentFsmagic,
    OldPath,
    OldPathIno,
    OldPathType,
    OldPathPerm,
    OldPathUid,
    OldPathGid,
    OldPathMajor,
    OldPathMinor,
    OldPathDevMajor,
    OldPathDevMinor,
    OldPathFsmagic,
    OldPathParent,
    OldPathParentType,
    OldPathParentIno,
    OldPathParentPerm,
    OldPathParentUid,
    OldPathParentGid,
    OldPathParentMajor,
    OldPathParentMinor,
    OldPathParentDevMajor,
    OldPathParentDevMinor,
    OldPathParentFsmagic,
    NewPath,
    NewPathIno,
    NewPathType,
    NewPathPerm,
    NewPathUid,
    NewPathGid,
    NewPathMajor,
    NewPathMinor,
    NewPathDevMajor,
    NewPathDevMinor,
    NewPathFsmagic,
    NewPathParent,
    NewPathParentIno,
    NewPathParentType,
    NewPathParentPerm,
    NewPathParentUid,
    NewPathParentGid,
    NewPathParentMajor,
    NewPathParentMinor,
    NewPathParentDevMajor,
    NewPathParentDevMinor,
    NewPathParentFsmagic,
    Source,
    SourceType,
    SourceIno,
    SourcePerm,
    SourceUid,
    SourceGid,
    SourceMajor,
    SourceMinor,
    SourceDevMajor,
    SourceDevMinor,
    SourceFsmagic,
    SourceParent,
    SourceParentIno,
    SourceParentType,
    SourceParentPerm,
    SourceParentUid,
    SourceParentGid,
    SourceParentMajor,
    SourceParentMinor,
    SourceParentDevMajor,
    SourceParentDevMinor,
    SourceParentFsmagic,
    Target,
    TargetType,
    TargetIno,
    TargetPerm,
    TargetUid,
    TargetGid,
    TargetMajor,
    TargetMinor,
    TargetDevMajor,
    TargetDevMinor,
    TargetFsmagic,
    TargetParent,
    TargetParentIno,
    TargetParentType,
    TargetParentPerm,
    TargetParentUid,
    TargetParentGid,
    TargetParentMajor,
    TargetParentMinor,
    TargetParentDevMajor,
    TargetParentDevMinor,
    TargetParentFsmagic,
    Fstype,
    IP,
    Port,
    Exec,
    ExecPid,
    ExecPpid,
    ExecUid,
    ExecEuid,
    ExecGid,
    ExecEgid,
    Transition,
    Error,
}

impl Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for Resource {
    fn from(res: &str) -> Self {
        match res {
            "task.pid" => Self::TaskPid,
            "task.ppid" => Self::TaskPpid,
            "task.uid" => Self::TaskUid,
            "task.euid" => Self::TaskEuid,
            "task.egid" => Self::TaskEgid,
            "task.suid" => Self::TaskSuid,
            "task.sgid" => Self::TaskSgid,
            "task.fsuid" => Self::TaskFsuid,
            "task.fsgid" => Self::TaskFsgid,
            "task.exe" => Self::TaskExe,
            "task.domain" => Self::TaskDomain,
            "task.type" => Self::TaskType,
            "perm" => Self::Perm,
            "path" => Self::Path,
            "path.ino" => Self::PathIno,
            "path.perm" => Self::PathPerm,
            "path.uid" => Self::PathUid,
            "path.gid" => Self::PathGid,
            "path.type" => Self::PathType,
            "path.dev_major" => Self::PathDevMajor,
            "path.major" => Self::PathMajor,
            "path.dev_minor" => Self::PathDevMinor,
            "path.minor" => Self::PathMinor,
            "path.fsmagic" => Self::PathFsmagic,
            "path.parent" => Self::PathParent,
            "path.parent.ino" => Self::PathParentIno,
            "path.parent.uid" => Self::PathParentUid,
            "path.parent.gid" => Self::PathParentGid,
            "path.parent.perm" => Self::PathParentPerm,
            "path.parent.type" => Self::PathParentType,
            "path.parent.dev_major" => Self::PathParentDevMajor,
            "path.parent.major" => Self::PathParentMajor,
            "path.parent.dev_minor" => Self::PathParentDevMinor,
            "path.parent.minor" => Self::PathParentMinor,
            "path.parent.fsmagic" => Self::PathParentFsmagic,
            "old_path" => Self::OldPath,
            "old_path.ino" => Self::OldPathIno,
            "old_path.perm" => Self::OldPathPerm,
            "old_path.uid" => Self::OldPathUid,
            "old_path.gid" => Self::OldPathGid,
            "old_path.type" => Self::OldPathType,
            "old_path.dev_major" => Self::OldPathDevMajor,
            "old_path.major" => Self::OldPathMajor,
            "old_path.dev_minor" => Self::OldPathDevMinor,
            "old_path.minor" => Self::OldPathMinor,
            "old_path.fsmagic" => Self::OldPathFsmagic,
            "old_path.parent" => Self::OldPathParent,
            "old_path.parent.ino" => Self::OldPathParentIno,
            "old_path.parent.uid" => Self::OldPathParentUid,
            "old_path.parent.gid" => Self::OldPathParentGid,
            "old_path.parent.perm" => Self::OldPathParentPerm,
            "old_path.parent.type" => Self::OldPathParentType,
            "old_path.parent.dev_major" => Self::OldPathParentDevMajor,
            "old_path.parent.major" => Self::OldPathParentMajor,
            "old_path.parent.dev_minor" => Self::OldPathParentDevMinor,
            "old_path.parent.minor" => Self::OldPathParentMinor,
            "old_path.parent.fsmagic" => Self::OldPathParentFsmagic,
            "new_path" => Self::NewPath,
            "new_path.ino" => Self::NewPathIno,
            "new_path.perm" => Self::NewPathPerm,
            "new_path.uid" => Self::NewPathUid,
            "new_path.gid" => Self::NewPathGid,
            "new_path.type" => Self::NewPathType,
            "new_path.dev_major" => Self::NewPathDevMajor,
            "new_path.major" => Self::NewPathMajor,
            "new_path.dev_minor" => Self::NewPathDevMinor,
            "new_path.minor" => Self::NewPathMinor,
            "new_path.fsmagic" => Self::NewPathFsmagic,
            "new_path.parent" => Self::NewPathParent,
            "new_path.parent.ino" => Self::NewPathParentIno,
            "new_path.parent.uid" => Self::NewPathParentUid,
            "new_path.parent.gid" => Self::NewPathParentGid,
            "new_path.parent.perm" => Self::NewPathParentPerm,
            "new_path.parent.type" => Self::NewPathParentType,
            "new_path.parent.dev_major" => Self::NewPathParentDevMajor,
            "new_path.parent.major" => Self::NewPathParentMajor,
            "new_path.parent.dev_minor" => Self::NewPathParentDevMinor,
            "new_path.parent.minor" => Self::NewPathParentMinor,
            "new_path.parent.fsmagic" => Self::NewPathParentFsmagic,
            "source" => Self::Source,
            "source.ino" => Self::SourceIno,
            "source.perm" => Self::SourcePerm,
            "source.uid" => Self::SourceUid,
            "source.gid" => Self::SourceGid,
            "source.type" => Self::SourceType,
            "source.dev_major" => Self::SourceDevMajor,
            "source.major" => Self::SourceMajor,
            "source.dev_minor" => Self::SourceDevMinor,
            "source.minor" => Self::SourceMinor,
            "source.fsmagic" => Self::SourceFsmagic,
            "source.parent" => Self::SourceParent,
            "source.parent.ino" => Self::SourceParentIno,
            "source.parent.uid" => Self::SourceParentUid,
            "source.parent.gid" => Self::SourceParentGid,
            "source.parent.perm" => Self::SourceParentPerm,
            "source.parent.type" => Self::SourceParentType,
            "source.parent.dev_major" => Self::SourceParentDevMajor,
            "source.parent.major" => Self::SourceParentMajor,
            "source.parent.dev_minor" => Self::SourceParentDevMinor,
            "source.parent.minor" => Self::SourceParentMinor,
            "source.parent.fsmagic" => Self::SourceParentFsmagic,
            "target" => Self::Target,
            "target.ino" => Self::TargetIno,
            "target.perm" => Self::TargetPerm,
            "target.uid" => Self::TargetUid,
            "target.gid" => Self::TargetGid,
            "target.type" => Self::TargetType,
            "target.dev_major" => Self::TargetDevMajor,
            "target.major" => Self::TargetMajor,
            "target.dev_minor" => Self::TargetDevMinor,
            "target.minor" => Self::TargetMinor,
            "target.fsmagic" => Self::TargetFsmagic,
            "target.parent" => Self::TargetParent,
            "target.parent.ino" => Self::TargetParentIno,
            "target.parent.uid" => Self::TargetParentUid,
            "target.parent.gid" => Self::TargetParentGid,
            "target.parent.perm" => Self::TargetParentPerm,
            "target.parent.type" => Self::TargetParentType,
            "target.parent.dev_major" => Self::TargetParentDevMajor,
            "target.parent.major" => Self::TargetParentMajor,
            "target.parent.dev_minor" => Self::TargetParentDevMinor,
            "target.parent.minor" => Self::TargetParentMinor,
            "target.parent.fsmagic" => Self::TargetParentFsmagic,
            "fstype" => Self::Fstype,
            "ip" => Self::IP,
            "port" => Self::Port,
            "exec" => Self::Exec,
            "exec.pid" => Self::ExecPid,
            "exec.ppid" => Self::ExecPpid,
            "exec.uid" => Self::ExecUid,
            "exec.euid" => Self::ExecEuid,
            "exec.gid" => Self::ExecGid,
            "exec.egid" => Self::ExecEgid,
            "transition" => Self::Transition,
            _ => Self::Error,
        }
    }
}

impl Resource {
    pub fn list() -> Vec<Resource> {
        vec![
            Resource::TaskPid,
            Resource::TaskPpid,
            Resource::TaskUid,
            Resource::TaskEuid,
            Resource::TaskSuid,
            Resource::TaskSgid,
            Resource::TaskFsuid,
            Resource::TaskFsgid,
            Resource::TaskExe,
            Resource::TaskDomain,
            Resource::TaskType,
            Resource::Perm,
            Resource::DevMajor,
            Resource::DevMinor,
            Resource::Path,
            Resource::OldPath,
            Resource::NewPath,
            Resource::Fstype,
            Resource::Path,
            Resource::PathPerm,
            Resource::PathType,
            Resource::PathUid,
            Resource::PathGid,
            Resource::PathMajor,
            Resource::PathMinor,
            Resource::PathDevMajor,
            Resource::PathDevMinor,
            Resource::PathFsmagic,
            Resource::PathParent,
            Resource::PathParentPerm,
            Resource::PathParentType,
            Resource::PathParentUid,
            Resource::PathParentGid,
            Resource::PathParentMajor,
            Resource::PathParentMinor,
            Resource::PathParentDevMajor,
            Resource::PathParentDevMinor,
            Resource::PathParentFsmagic,
            Resource::OldPath,
            Resource::OldPathPerm,
            Resource::OldPathType,
            Resource::OldPathUid,
            Resource::OldPathGid,
            Resource::OldPathMajor,
            Resource::OldPathMinor,
            Resource::OldPathDevMajor,
            Resource::OldPathDevMinor,
            Resource::OldPathFsmagic,
            Resource::OldPathParent,
            Resource::OldPathParentPerm,
            Resource::OldPathParentType,
            Resource::OldPathParentUid,
            Resource::OldPathParentGid,
            Resource::OldPathParentMajor,
            Resource::OldPathParentMinor,
            Resource::OldPathParentDevMajor,
            Resource::OldPathParentDevMinor,
            Resource::OldPathParentFsmagic,
            Resource::NewPath,
            Resource::NewPathPerm,
            Resource::NewPathType,
            Resource::NewPathUid,
            Resource::NewPathGid,
            Resource::NewPathMajor,
            Resource::NewPathMinor,
            Resource::NewPathDevMajor,
            Resource::NewPathDevMinor,
            Resource::NewPathFsmagic,
            Resource::NewPathParent,
            Resource::NewPathParentPerm,
            Resource::NewPathParentType,
            Resource::NewPathParentUid,
            Resource::NewPathParentGid,
            Resource::NewPathParentMajor,
            Resource::NewPathParentMinor,
            Resource::NewPathParentDevMajor,
            Resource::NewPathParentDevMinor,
            Resource::NewPathParentFsmagic,
            Resource::Source,
            Resource::SourcePerm,
            Resource::SourceType,
            Resource::SourceUid,
            Resource::SourceGid,
            Resource::SourceMajor,
            Resource::SourceMinor,
            Resource::SourceDevMajor,
            Resource::SourceDevMinor,
            Resource::SourceFsmagic,
            Resource::SourceParent,
            Resource::SourceParentPerm,
            Resource::SourceParentType,
            Resource::SourceParentUid,
            Resource::SourceParentGid,
            Resource::SourceParentMajor,
            Resource::SourceParentMinor,
            Resource::SourceParentDevMajor,
            Resource::SourceParentDevMinor,
            Resource::SourceParentFsmagic,
            Resource::Target,
            Resource::TargetPerm,
            Resource::TargetType,
            Resource::TargetUid,
            Resource::TargetGid,
            Resource::TargetMajor,
            Resource::TargetMinor,
            Resource::TargetDevMajor,
            Resource::TargetDevMinor,
            Resource::TargetFsmagic,
            Resource::TargetParent,
            Resource::TargetParentPerm,
            Resource::TargetParentType,
            Resource::TargetParentUid,
            Resource::TargetParentGid,
            Resource::TargetParentMajor,
            Resource::TargetParentMinor,
            Resource::TargetParentDevMajor,
            Resource::TargetParentDevMinor,
            Resource::TargetParentFsmagic,
            Resource::IP,
            Resource::Port,
            Resource::Exec,
            Resource::ExecPid,
            Resource::ExecPpid,
            Resource::ExecUid,
            Resource::ExecEuid,
            Resource::ExecGid,
            Resource::ExecEgid,
            Resource::Transition,
        ]
    }
    pub fn list_str() -> Vec<&'static str> {
        let mut res = vec![];
        for r in Resource::list() {
            res.push(r.as_str());
        }
        res
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TaskPid => "task.pid",
            Self::TaskPpid => "task.ppid",
            Self::TaskUid => "task.uid",
            Self::TaskEuid => "task.euid",
            Self::TaskEgid => "task.egid",
            Self::TaskSuid => "task.suid",
            Self::TaskFsuid => "task.fsuid",
            Self::TaskSgid => "task.sgid",
            Self::TaskFsgid => "task.fsgid",
            Self::TaskExe => "task.exe",
            Self::TaskDomain => "task.domain",
            Self::TaskType => "task.type",
            Self::Source => "source",
            Self::SourceIno => "source.ino",
            Self::SourcePerm => "source.perm",
            Self::SourceType => "source.type",
            Self::SourceUid => "source.uid",
            Self::SourceGid => "source.gid",
            Self::SourceMajor => "source.major",
            Self::SourceMinor => "source.minor",
            Self::SourceDevMajor => "source.dev_major",
            Self::SourceDevMinor => "source.dev_minor",
            Self::SourceFsmagic => "source.fsmagic",
            Self::SourceParent => "source.parent",
            Self::SourceParentIno => "source.parent.ino",
            Self::SourceParentPerm => "source.parent.perm",
            Self::SourceParentType => "source.parent.type",
            Self::SourceParentUid => "source.parent.uid",
            Self::SourceParentGid => "source.parent.gid",
            Self::SourceParentMajor => "source.parent.major",
            Self::SourceParentMinor => "source.parent.minor",
            Self::SourceParentDevMajor => "source.parent.dev_major",
            Self::SourceParentDevMinor => "source.parent.dev_minor",
            Self::SourceParentFsmagic => "source.parent.fsmagic",
            Self::Target => "target",
            Self::TargetIno => "target.ino",
            Self::TargetPerm => "target.perm",
            Self::TargetType => "target.type",
            Self::TargetUid => "target.uid",
            Self::TargetGid => "target.gid",
            Self::TargetMajor => "target.major",
            Self::TargetMinor => "target.minor",
            Self::TargetDevMajor => "target.dev_major",
            Self::TargetDevMinor => "target.dev_minor",
            Self::TargetFsmagic => "target.fsmagic",
            Self::TargetParent => "target.parent",
            Self::TargetParentIno => "target.parent.ino",
            Self::TargetParentPerm => "target.parent.perm",
            Self::TargetParentType => "target.parent.type",
            Self::TargetParentUid => "target.parent.uid",
            Self::TargetParentGid => "target.parent.gid",
            Self::TargetParentMajor => "target.parent.major",
            Self::TargetParentMinor => "target.parent.minor",
            Self::TargetParentDevMajor => "target.parent.dev_major",
            Self::TargetParentDevMinor => "target.parent.dev_minor",
            Self::TargetParentFsmagic => "target.parent.fsmagic",
            Self::Fstype => "fstype",
            Self::Perm => "perm",
            Self::DevMajor => "dev_major",
            Self::DevMinor => "dev_minor",
            Self::Path => "path",
            Self::PathIno => "path.ino",
            Self::PathPerm => "path.perm",
            Self::PathType => "path.type",
            Self::PathUid => "path.uid",
            Self::PathGid => "path.gid",
            Self::PathMajor => "path.major",
            Self::PathMinor => "path.minor",
            Self::PathDevMajor => "path.dev_major",
            Self::PathDevMinor => "path.dev_minor",
            Self::PathFsmagic => "path.fsmagic",
            Self::PathParent => "path.parent",
            Self::PathParentIno => "path.parent.ino",
            Self::PathParentPerm => "path.parent.perm",
            Self::PathParentType => "path.parent.type",
            Self::PathParentUid => "path.parent.uid",
            Self::PathParentGid => "path.parent.gid",
            Self::PathParentMajor => "path.parent.major",
            Self::PathParentMinor => "path.parent.minor",
            Self::PathParentDevMajor => "path.parent.dev_major",
            Self::PathParentDevMinor => "path.parent.dev_minor",
            Self::PathParentFsmagic => "path.parent.fsmagic",
            Self::OldPath => "old_path",
            Self::OldPathIno => "old_path.ino",
            Self::OldPathPerm => "old_path.perm",
            Self::OldPathType => "old_path.type",
            Self::OldPathUid => "old_path.uid",
            Self::OldPathGid => "old_path.gid",
            Self::OldPathMajor => "old_path.major",
            Self::OldPathMinor => "old_path.minor",
            Self::OldPathDevMajor => "old_path.dev_major",
            Self::OldPathDevMinor => "old_path.dev_minor",
            Self::OldPathParent => "old_path.parent",
            Self::OldPathFsmagic => "old_path.fsmagic",
            Self::OldPathParentIno => "old_path.parent.ino",
            Self::OldPathParentPerm => "old_path.parent.perm",
            Self::OldPathParentType => "old_path.parent.type",
            Self::OldPathParentUid => "old_path.parent.uid",
            Self::OldPathParentGid => "old_path.parent.gid",
            Self::OldPathParentMajor => "old_path.parent.major",
            Self::OldPathParentMinor => "old_path.parent.minor",
            Self::OldPathParentDevMajor => "old_path.parent.dev_major",
            Self::OldPathParentDevMinor => "old_path.parent.dev_minor",
            Self::OldPathParentFsmagic => "old_path.parent.fsmagic",
            Self::NewPath => "new_path",
            Self::NewPathIno => "new_path.ino",
            Self::NewPathPerm => "new_path.perm",
            Self::NewPathType => "new_path.type",
            Self::NewPathUid => "new_path.uid",
            Self::NewPathGid => "new_path.gid",
            Self::NewPathMajor => "new_path.major",
            Self::NewPathMinor => "new_path.minor",
            Self::NewPathDevMajor => "new_path.dev_major",
            Self::NewPathDevMinor => "new_path.dev_minor",
            Self::NewPathFsmagic => "new_path.fsmagic",
            Self::NewPathParent => "new_path.parent",
            Self::NewPathParentIno => "new_path.parent.ino",
            Self::NewPathParentPerm => "new_path.parent.perm",
            Self::NewPathParentType => "new_path.parent.type",
            Self::NewPathParentUid => "new_path.parent.uid",
            Self::NewPathParentGid => "new_path.parent.gid",
            Self::NewPathParentMajor => "new_path.parent.major",
            Self::NewPathParentMinor => "new_path.parent.minor",
            Self::NewPathParentDevMajor => "new_path.parent.dev_major",
            Self::NewPathParentDevMinor => "new_path.parent.dev_minor",
            Self::NewPathParentFsmagic => "new_path.parent.fsmagic",
            Self::IP => "ip",
            Self::Port => "port",
            Self::Exec => "exec",
            Self::ExecPid => "exec.pid",
            Self::ExecPpid => "exec.ppid",
            Self::ExecUid => "exec.uid",
            Self::ExecEuid => "exec.euid",
            Self::ExecGid => "exec.gid",
            Self::ExecEgid => "exec.egid",
            Self::Transition => "transition",
            Self::Error => "",
        }
    }
}

/// Cond represents condition matcher (= or !=) for attributes.
///
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Cond {
    Eq,
    Ne,
    Error,
}

impl From<&str> for Cond {
    fn from(res: &str) -> Self {
        match res {
            "=" => Self::Eq,
            "!=" => Self::Ne,
            _ => Self::Error,
        }
    }
}

impl Display for Cond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Cond {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Error => "",
        }
    }
}
