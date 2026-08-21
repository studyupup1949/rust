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

        let attr_list = String::from(r"(?P<attrs>(\s+") + attribute_pattern.clone().as_str() + ")*)";
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
    ///let c = m.get_category("1000 acl read");
    ///assert_eq!(c == Category::Header, true);
    ///```
    pub fn get_category(&self, line: &str) -> Category {
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

    pub fn get_version(&self, line: &str) -> u32 {
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

    pub fn get_number_group(&self, line: &str) -> (String, String) {
        let mat = self.number_group.captures(&line).unwrap();
        (mat[1].to_string(), mat[2].to_string())
    }

    pub fn is_ip_group(&self, line: &str) -> bool {
        self.ip_group.is_match(line)
    }

    pub fn get_ip_group(&self, line: &str) -> (String, String) {
        let mat = self.ip_group.captures(&line).unwrap();
        (mat[1].to_string(), mat[2].to_string())
    }

    pub fn is_string_group(&self, line: &str) -> bool {
        self.string_group.is_match(line)
    }

    pub fn get_string_group(&self, line: &str) -> (String, String) {
        let mat = self.string_group.captures(&line).unwrap();
        (mat[1].to_string(), mat[2].to_string())
    }

    pub fn is_acl_header(&self, line: &str) -> bool {
        self.acl_header.is_match(line)
    }

    /// get_acl_header returns acl header properties within a tuple.
    /// Returned tuple holds acl priority, Operation and optional attribute for
    /// the Operation.
    ///
    pub fn get_acl_header(&self, line: &str) -> (u32, Op, Vec<(Resource, Cond, String)>) {
        let cap = &self.acl_header.captures(line).unwrap();
        (
            cap["priority"].to_string().parse::<u32>().unwrap(),
            Op::from(&cap["operation"]),
            self.get_attr_list(&cap["attrs"].to_string()),
        )
    }

    /// get_attr_list returns a list of attributes which conposes optional ACL modifier.
    ///
    pub fn get_attr_list(&self, line: &str) -> Vec<(Resource, Cond, String)> {
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

    /// get_acl_audit_sequence returns audit sequence value
    /// set to an audit line.
    /// Any lines supplied to this function must be a valid
    /// audit lines which is parsable with self.acl_audit matcher.
    ///
    pub fn get_acl_audit_sequence(&self, line: &str) -> u32 {
        let cap = &self.acl_audit.captures(line).unwrap();
        cap["seq"].to_string().parse::<u32>().unwrap()
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

    /// get_acl_rule returns ACL rule properties within a tuple,
    /// which contains rule priority, verb (allow/deny) and attribute
    /// list in vector format.
    /// Any lines supplied to this function must be a valid rule line
    /// which is ensured to be parsed with self.acl_rule matcher.
    ///
    pub fn get_acl_rule(&self, line: &str) -> (u32, Verb, Vec<(Resource, Cond, String)>) {
        let cap = &self.acl_rule.captures(line).unwrap();
        (
            cap["priority"].to_string().parse::<u32>().unwrap(),
            Verb::from(&cap["verb"]),
            self.get_attr_list(&cap["attrs"].to_string()),
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
    pub fn get_header(&self, line: &str) -> Option<String> {
        match self.header.captures(&line) {
            Some(cap) => Some(String::from(&cap[0])),
            None => None,
        }
    }
    pub fn get_body(&self, line: &str) -> Option<String> {
        match self.body.captures(&line) {
            Some(cap) => Some(String::from(&cap["body"])),
            None => None,
        }
    }
    pub fn is_operation(&self, line: &str) -> bool {
        self.operation.is_match(line)
    }
    pub fn get_operation(&self, line: &str) -> Option<String> {
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
    TaskExe,
    TaskDomain,
    Path,
    OldPath,
    NewPath,
    Source,
    Target,
    Fstype,
    PathPerm,
    PathUid,
    PathGid,
    PathParentUid,
    PathParentGid,
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
            "task.exe" => Self::TaskExe,
            "task.domain" => Self::TaskDomain,
            "path" => Self::Path,
            "old_path" => Self::OldPath,
            "new_path" => Self::NewPath,
            "source" => Self::Source,
            "target" => Self::Target,
            "fstype" => Self::Fstype,
            "path.perm" => Self::PathPerm,
            "path.uid" => Self::PathUid,
            "path.gid" => Self::PathGid,
            "path.parent.uid" => Self::PathParentUid,
            "path.parent.gid" => Self::PathParentGid,
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
            Resource::TaskExe,
            Resource::TaskDomain,
            Resource::Path,
            Resource::OldPath,
            Resource::NewPath,
            Resource::Source,
            Resource::Target,
            Resource::Fstype,
            Resource::PathPerm,
            Resource::PathUid,
            Resource::PathGid,
            Resource::PathParentUid,
            Resource::PathParentGid,
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
            Self::TaskExe => "task.exe",
            Self::TaskDomain => "task.domain",
            Self::Path => "path",
            Self::OldPath => "old_path",
            Self::NewPath => "new_path",
            Self::Source => "source",
            Self::Target => "target",
            Self::Fstype => "fstype",
            Self::PathPerm => "path.perm",
            Self::PathUid => "path.uid",
            Self::PathGid => "path.gid",
            Self::PathParentUid => "path.parent.uid",
            Self::PathParentGid => "path.parent.gid",
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
