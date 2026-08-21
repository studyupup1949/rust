use ::windows::Win32::{
    Networking::ActiveDirectory::{
        ACTRL_DS_CONTROL_ACCESS,
        ACTRL_DS_CREATE_CHILD,
        ACTRL_DS_DELETE_CHILD,
        ACTRL_DS_DELETE_TREE,
        ACTRL_DS_LIST,
        ACTRL_DS_LIST_OBJECT,
        ACTRL_DS_OPEN,
        ACTRL_DS_READ_PROP,
        ACTRL_DS_SELF,
        ACTRL_DS_WRITE_PROP,
    },
    Security::{
        TOKEN_ACCESS_PSEUDO_HANDLE,
        TOKEN_ACCESS_PSEUDO_HANDLE_WIN8,
        TOKEN_ACCESS_SYSTEM_SECURITY,
        TOKEN_ADJUST_DEFAULT,
        TOKEN_ADJUST_GROUPS,
        TOKEN_ADJUST_PRIVILEGES,
        TOKEN_ADJUST_SESSIONID,
        TOKEN_ALL_ACCESS,
        TOKEN_ASSIGN_PRIMARY,
        TOKEN_DUPLICATE,
        TOKEN_EXECUTE,
        TOKEN_IMPERSONATE,
        TOKEN_QUERY,
        TOKEN_QUERY_SOURCE,
        TOKEN_READ,
        TOKEN_TRUST_CONSTRAINT_MASK,
        TOKEN_WRITE,
    },
    Storage::FileSystem::{
        FILE_ALL_ACCESS,
        FILE_APPEND_DATA,
        FILE_DELETE_CHILD,
        FILE_GENERIC_EXECUTE,
        FILE_GENERIC_READ,
        FILE_GENERIC_WRITE,
        FILE_READ_ATTRIBUTES,
        FILE_READ_DATA,
        FILE_READ_EA,
        FILE_WRITE_ATTRIBUTES,
        FILE_WRITE_DATA,
        FILE_WRITE_EA,
        STANDARD_RIGHTS_REQUIRED,
        SYNCHRONIZE,
    },
    System::{
        Registry::{
            KEY_ALL_ACCESS,
            KEY_CREATE_LINK,
            KEY_CREATE_SUB_KEY,
            KEY_ENUMERATE_SUB_KEYS,
            KEY_EXECUTE,
            KEY_NOTIFY,
            KEY_QUERY_VALUE,
            KEY_READ,
            KEY_SET_VALUE,
            KEY_WOW64_32KEY,
            KEY_WOW64_64KEY,
            KEY_WOW64_RES,
            KEY_WRITE,
        },
        StationsAndDesktops::{
            DESKTOP_CREATEMENU,
            DESKTOP_CREATEWINDOW,
            DESKTOP_ENUMERATE,
            DESKTOP_HOOKCONTROL,
            DESKTOP_JOURNALPLAYBACK,
            DESKTOP_JOURNALRECORD,
            DESKTOP_READOBJECTS,
            DESKTOP_SWITCHDESKTOP,
            DESKTOP_WRITEOBJECTS,
        },
        SystemServices::{
            JOB_OBJECT_ASSIGN_PROCESS,
            JOB_OBJECT_QUERY,
            JOB_OBJECT_SET_ATTRIBUTES,
            JOB_OBJECT_SET_SECURITY_ATTRIBUTES,
            JOB_OBJECT_TERMINATE,
            MUTANT_QUERY_STATE,
        },
        Threading::{
            EVENT_ALL_ACCESS,
            EVENT_MODIFY_STATE,
            MUTEX_ALL_ACCESS,
            MUTEX_MODIFY_STATE,
            PROCESS_ALL_ACCESS,
            PROCESS_CREATE_PROCESS,
            PROCESS_CREATE_THREAD,
            PROCESS_DUP_HANDLE,
            PROCESS_QUERY_INFORMATION,
            PROCESS_QUERY_LIMITED_INFORMATION,
            PROCESS_SET_INFORMATION,
            PROCESS_SET_LIMITED_INFORMATION,
            PROCESS_SET_QUOTA,
            PROCESS_SET_SESSIONID,
            PROCESS_STANDARD_RIGHTS_REQUIRED,
            PROCESS_SUSPEND_RESUME,
            PROCESS_TERMINATE,
            PROCESS_VM_OPERATION,
            PROCESS_VM_READ,
            PROCESS_VM_WRITE,
            SEMAPHORE_ALL_ACCESS,
            SEMAPHORE_MODIFY_STATE,
            THREAD_ALL_ACCESS,
            THREAD_DIRECT_IMPERSONATION,
            THREAD_GET_CONTEXT,
            THREAD_IMPERSONATE,
            THREAD_QUERY_LIMITED_INFORMATION,
            THREAD_RESUME,
            THREAD_SET_CONTEXT,
            THREAD_SET_INFORMATION,
            THREAD_SET_LIMITED_INFORMATION,
            THREAD_SET_THREAD_TOKEN,
            THREAD_STANDARD_RIGHTS_REQUIRED,
            THREAD_SUSPEND_RESUME,
            THREAD_TERMINATE,
            TIMER_ALL_ACCESS,
            TIMER_MODIFY_STATE,
            TIMER_QUERY_STATE,
        },
    },
    UI::WindowsAndMessaging::{
        WINSTA_ACCESSCLIPBOARD,
        WINSTA_ACCESSGLOBALATOMS,
        WINSTA_ALL_ACCESS,
        WINSTA_CREATEDESKTOP,
        WINSTA_ENUMDESKTOPS,
        WINSTA_ENUMERATE,
        WINSTA_EXITWINDOWS,
        WINSTA_READATTRIBUTES,
        WINSTA_READSCREEN,
        WINSTA_WRITEATTRIBUTES,
    },
};
use clap::{Parser, ValueEnum};
use std::fmt::Display;

const JOB_OBJECT_ALL_ACCESS: u32 = 0x001F_001F;
const PORT_CONNECT: u32 = 0x0001;
const PORT_ALL_ACCESS: u32 = STANDARD_RIGHTS_REQUIRED.0 | SYNCHRONIZE.0 | 0x1;

#[derive(Debug, Clone, Eq, PartialEq, ValueEnum)]
pub enum Type {
    Process,
    Thread,
    Event,
    Mutex,
    Mutant,
    Semaphore,
    Timer,
    Desktop,
    WindowsStation,
    Winsta,
    Key,
    Token,
    Job,
    File,
    Directory,
    Alpc,
    Port,
    ActiveDirectory,
}

impl Type {
    #[allow(clippy::too_many_lines)]
    pub fn print_parsed_type(&self, value: u32) {
        println!("Specific rights ({self}):");
        match self {
            Type::Process => {
                crate::print_attribute!(value, PROCESS_ALL_ACCESS, PROCESS_ALL_ACCESS.0);
                crate::print_attribute!(value, PROCESS_CREATE_PROCESS, PROCESS_CREATE_PROCESS.0);
                crate::print_attribute!(value, PROCESS_CREATE_THREAD, PROCESS_CREATE_THREAD.0);
                crate::print_attribute!(value, PROCESS_DUP_HANDLE, PROCESS_DUP_HANDLE.0);
                crate::print_attribute!(
                    value,
                    PROCESS_QUERY_INFORMATION,
                    PROCESS_QUERY_INFORMATION.0
                );
                crate::print_attribute!(
                    value,
                    PROCESS_QUERY_LIMITED_INFORMATION,
                    PROCESS_QUERY_LIMITED_INFORMATION.0
                );
                crate::print_attribute!(value, PROCESS_SET_INFORMATION, PROCESS_SET_INFORMATION.0);
                crate::print_attribute!(
                    value,
                    PROCESS_SET_LIMITED_INFORMATION,
                    PROCESS_SET_LIMITED_INFORMATION.0
                );
                crate::print_attribute!(value, PROCESS_SET_QUOTA, PROCESS_SET_QUOTA.0);
                crate::print_attribute!(value, PROCESS_SET_SESSIONID, PROCESS_SET_SESSIONID.0);
                crate::print_attribute!(
                    value,
                    PROCESS_STANDARD_RIGHTS_REQUIRED,
                    PROCESS_STANDARD_RIGHTS_REQUIRED.0
                );
                crate::print_attribute!(value, PROCESS_SUSPEND_RESUME, PROCESS_SUSPEND_RESUME.0);
                crate::print_attribute!(value, PROCESS_TERMINATE, PROCESS_TERMINATE.0);
                crate::print_attribute!(value, PROCESS_VM_OPERATION, PROCESS_VM_OPERATION.0);
                crate::print_attribute!(value, PROCESS_VM_READ, PROCESS_VM_READ.0);
                crate::print_attribute!(value, PROCESS_VM_WRITE, PROCESS_VM_WRITE.0);
            }
            Type::Thread => {
                crate::print_attribute!(value, THREAD_ALL_ACCESS, THREAD_ALL_ACCESS.0);
                crate::print_attribute!(
                    value,
                    THREAD_DIRECT_IMPERSONATION,
                    THREAD_DIRECT_IMPERSONATION.0
                );
                crate::print_attribute!(value, THREAD_GET_CONTEXT, THREAD_GET_CONTEXT.0);
                crate::print_attribute!(value, THREAD_IMPERSONATE, THREAD_IMPERSONATE.0);
                crate::print_attribute!(
                    value,
                    THREAD_QUERY_LIMITED_INFORMATION,
                    THREAD_QUERY_LIMITED_INFORMATION.0
                );
                crate::print_attribute!(value, THREAD_RESUME, THREAD_RESUME.0);
                crate::print_attribute!(value, THREAD_SET_CONTEXT, THREAD_SET_CONTEXT.0);
                crate::print_attribute!(value, THREAD_SET_INFORMATION, THREAD_SET_INFORMATION.0);
                crate::print_attribute!(
                    value,
                    THREAD_SET_LIMITED_INFORMATION,
                    THREAD_SET_LIMITED_INFORMATION.0
                );
                crate::print_attribute!(value, THREAD_SET_THREAD_TOKEN, THREAD_SET_THREAD_TOKEN.0);
                crate::print_attribute!(
                    value,
                    THREAD_STANDARD_RIGHTS_REQUIRED,
                    THREAD_STANDARD_RIGHTS_REQUIRED.0
                );
                crate::print_attribute!(value, THREAD_SUSPEND_RESUME, THREAD_SUSPEND_RESUME.0);
                crate::print_attribute!(value, THREAD_TERMINATE, THREAD_TERMINATE.0);
            }
            Type::Event => {
                crate::print_attribute!(value, EVENT_ALL_ACCESS, EVENT_ALL_ACCESS.0);
                crate::print_attribute!(value, EVENT_MODIFY_STATE, EVENT_MODIFY_STATE.0);
            }
            Type::Mutex | Type::Mutant => {
                crate::print_attribute!(value, MUTANT_QUERY_STATE, MUTANT_QUERY_STATE);
                crate::print_attribute!(value, MUTEX_ALL_ACCESS, MUTEX_ALL_ACCESS.0);
                crate::print_attribute!(value, MUTEX_MODIFY_STATE, MUTEX_MODIFY_STATE.0);
            }
            Type::Semaphore => {
                crate::print_attribute!(value, SEMAPHORE_ALL_ACCESS, SEMAPHORE_ALL_ACCESS.0);
                crate::print_attribute!(value, SEMAPHORE_MODIFY_STATE, SEMAPHORE_MODIFY_STATE.0);
            }
            Type::Timer => {
                crate::print_attribute!(value, TIMER_ALL_ACCESS, TIMER_ALL_ACCESS.0);
                crate::print_attribute!(value, TIMER_MODIFY_STATE, TIMER_MODIFY_STATE.0);
                crate::print_attribute!(value, TIMER_QUERY_STATE, TIMER_QUERY_STATE.0);
            }
            Type::Desktop => {
                crate::print_attribute!(value, DESKTOP_CREATEMENU, DESKTOP_CREATEMENU.0);
                crate::print_attribute!(value, DESKTOP_CREATEWINDOW, DESKTOP_CREATEWINDOW.0);
                crate::print_attribute!(value, DESKTOP_ENUMERATE, DESKTOP_ENUMERATE.0);
                crate::print_attribute!(value, DESKTOP_HOOKCONTROL, DESKTOP_HOOKCONTROL.0);
                crate::print_attribute!(value, DESKTOP_JOURNALPLAYBACK, DESKTOP_JOURNALPLAYBACK.0);
                crate::print_attribute!(value, DESKTOP_JOURNALRECORD, DESKTOP_JOURNALRECORD.0);
                crate::print_attribute!(value, DESKTOP_READOBJECTS, DESKTOP_READOBJECTS.0);
                crate::print_attribute!(value, DESKTOP_SWITCHDESKTOP, DESKTOP_SWITCHDESKTOP.0);
                crate::print_attribute!(value, DESKTOP_WRITEOBJECTS, DESKTOP_WRITEOBJECTS.0);
            }
            Type::WindowsStation | Type::Winsta => {
                crate::print_attribute!(value, WINSTA_ACCESSCLIPBOARD, WINSTA_ACCESSCLIPBOARD);
                crate::print_attribute!(value, WINSTA_ACCESSGLOBALATOMS, WINSTA_ACCESSGLOBALATOMS);
                crate::print_attribute!(value, WINSTA_ALL_ACCESS, WINSTA_ALL_ACCESS);
                crate::print_attribute!(value, WINSTA_CREATEDESKTOP, WINSTA_CREATEDESKTOP);
                crate::print_attribute!(value, WINSTA_ENUMDESKTOPS, WINSTA_ENUMDESKTOPS);
                crate::print_attribute!(value, WINSTA_ENUMERATE, WINSTA_ENUMERATE);
                crate::print_attribute!(value, WINSTA_EXITWINDOWS, WINSTA_EXITWINDOWS);
                crate::print_attribute!(value, WINSTA_READATTRIBUTES, WINSTA_READATTRIBUTES);
                crate::print_attribute!(value, WINSTA_READSCREEN, WINSTA_READSCREEN);
                crate::print_attribute!(value, WINSTA_WRITEATTRIBUTES, WINSTA_WRITEATTRIBUTES);
            }
            Type::Key => {
                crate::print_attribute!(value, KEY_ALL_ACCESS, KEY_ALL_ACCESS.0);
                crate::print_attribute!(value, KEY_CREATE_LINK, KEY_CREATE_LINK.0);
                crate::print_attribute!(value, KEY_CREATE_SUB_KEY, KEY_CREATE_SUB_KEY.0);
                crate::print_attribute!(value, KEY_ENUMERATE_SUB_KEYS, KEY_ENUMERATE_SUB_KEYS.0);
                crate::print_attribute!(value, KEY_EXECUTE, KEY_EXECUTE.0);
                crate::print_attribute!(value, KEY_NOTIFY, KEY_NOTIFY.0);
                crate::print_attribute!(value, KEY_QUERY_VALUE, KEY_QUERY_VALUE.0);
                crate::print_attribute!(value, KEY_READ, KEY_READ.0);
                crate::print_attribute!(value, KEY_SET_VALUE, KEY_SET_VALUE.0);
                crate::print_attribute!(value, KEY_WOW64_32KEY, KEY_WOW64_32KEY.0);
                crate::print_attribute!(value, KEY_WOW64_64KEY, KEY_WOW64_64KEY.0);
                crate::print_attribute!(value, KEY_WOW64_RES, KEY_WOW64_RES.0);
                crate::print_attribute!(value, KEY_WRITE, KEY_WRITE.0);
            }
            Type::Token => {
                crate::print_attribute!(
                    value,
                    TOKEN_ACCESS_PSEUDO_HANDLE,
                    TOKEN_ACCESS_PSEUDO_HANDLE.0
                );
                crate::print_attribute!(
                    value,
                    TOKEN_ACCESS_PSEUDO_HANDLE_WIN8,
                    TOKEN_ACCESS_PSEUDO_HANDLE_WIN8.0
                );
                crate::print_attribute!(
                    value,
                    TOKEN_ACCESS_SYSTEM_SECURITY,
                    TOKEN_ACCESS_SYSTEM_SECURITY.0
                );
                crate::print_attribute!(value, TOKEN_ADJUST_DEFAULT, TOKEN_ADJUST_DEFAULT.0);
                crate::print_attribute!(value, TOKEN_ADJUST_GROUPS, TOKEN_ADJUST_GROUPS.0);
                crate::print_attribute!(value, TOKEN_ADJUST_PRIVILEGES, TOKEN_ADJUST_PRIVILEGES.0);
                crate::print_attribute!(value, TOKEN_ADJUST_SESSIONID, TOKEN_ADJUST_SESSIONID.0);
                crate::print_attribute!(value, TOKEN_ALL_ACCESS, TOKEN_ALL_ACCESS.0);
                crate::print_attribute!(value, TOKEN_ASSIGN_PRIMARY, TOKEN_ASSIGN_PRIMARY.0);
                crate::print_attribute!(value, TOKEN_DUPLICATE, TOKEN_DUPLICATE.0);
                crate::print_attribute!(value, TOKEN_EXECUTE, TOKEN_EXECUTE.0);
                crate::print_attribute!(value, TOKEN_IMPERSONATE, TOKEN_IMPERSONATE.0);
                crate::print_attribute!(value, TOKEN_QUERY, TOKEN_QUERY.0);
                crate::print_attribute!(value, TOKEN_QUERY_SOURCE, TOKEN_QUERY_SOURCE.0);
                crate::print_attribute!(value, TOKEN_READ, TOKEN_READ.0);
                crate::print_attribute!(
                    value,
                    TOKEN_TRUST_CONSTRAINT_MASK,
                    TOKEN_TRUST_CONSTRAINT_MASK.0
                );
                crate::print_attribute!(value, TOKEN_WRITE, TOKEN_WRITE.0);
            }
            Type::Job => {
                crate::print_attribute!(
                    value,
                    JOB_OBJECT_ASSIGN_PROCESS,
                    JOB_OBJECT_ASSIGN_PROCESS
                );
                crate::print_attribute!(
                    value,
                    JOB_OBJECT_SET_ATTRIBUTES,
                    JOB_OBJECT_SET_ATTRIBUTES
                );
                crate::print_attribute!(value, JOB_OBJECT_QUERY, JOB_OBJECT_QUERY);
                crate::print_attribute!(value, JOB_OBJECT_TERMINATE, JOB_OBJECT_TERMINATE);
                crate::print_attribute!(
                    value,
                    JOB_OBJECT_SET_SECURITY_ATTRIBUTES,
                    JOB_OBJECT_SET_SECURITY_ATTRIBUTES
                );
                crate::print_attribute!(value, JOB_OBJECT_ALL_ACCESS, JOB_OBJECT_ALL_ACCESS);
            }
            Type::File | Type::Directory => {
                crate::print_attribute!(value, FILE_GENERIC_READ, FILE_GENERIC_READ.0);
                crate::print_attribute!(value, FILE_READ_ATTRIBUTES, FILE_READ_ATTRIBUTES.0);
                crate::print_attribute!(value, FILE_READ_DATA, FILE_READ_DATA.0);
                crate::print_attribute!(value, FILE_READ_EA, FILE_READ_EA.0);
                crate::print_attribute!(value, FILE_GENERIC_WRITE, FILE_GENERIC_WRITE.0);
                crate::print_attribute!(value, FILE_WRITE_ATTRIBUTES, FILE_WRITE_ATTRIBUTES.0);
                crate::print_attribute!(value, FILE_WRITE_DATA, FILE_WRITE_DATA.0);
                crate::print_attribute!(value, FILE_WRITE_EA, FILE_WRITE_EA.0);
                crate::print_attribute!(value, FILE_APPEND_DATA, FILE_APPEND_DATA.0);
                crate::print_attribute!(value, FILE_GENERIC_EXECUTE, FILE_GENERIC_EXECUTE.0);
                crate::print_attribute!(value, FILE_DELETE_CHILD, FILE_DELETE_CHILD.0);
                crate::print_attribute!(value, FILE_ALL_ACCESS, FILE_ALL_ACCESS.0);
            }
            Type::Alpc | Type::Port => {
                crate::print_attribute!(value, PORT_CONNECT, PORT_CONNECT);
                crate::print_attribute!(value, PORT_ALL_ACCESS, PORT_ALL_ACCESS);
            }
            Type::ActiveDirectory => {
                crate::print_attribute!(value, ACTRL_DS_CONTROL_ACCESS, ACTRL_DS_CONTROL_ACCESS);
                crate::print_attribute!(value, ACTRL_DS_CREATE_CHILD, ACTRL_DS_CREATE_CHILD);
                crate::print_attribute!(value, ACTRL_DS_DELETE_CHILD, ACTRL_DS_DELETE_CHILD);
                crate::print_attribute!(value, ACTRL_DS_DELETE_TREE, ACTRL_DS_DELETE_TREE);
                crate::print_attribute!(value, ACTRL_DS_LIST, ACTRL_DS_LIST);
                crate::print_attribute!(value, ACTRL_DS_LIST_OBJECT, ACTRL_DS_LIST_OBJECT);
                crate::print_attribute!(value, ACTRL_DS_OPEN, ACTRL_DS_OPEN);
                crate::print_attribute!(value, ACTRL_DS_READ_PROP, ACTRL_DS_READ_PROP);
                crate::print_attribute!(value, ACTRL_DS_SELF, ACTRL_DS_SELF);
                crate::print_attribute!(value, ACTRL_DS_WRITE_PROP, ACTRL_DS_WRITE_PROP);
            }
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Process => write!(f, "process"),
            Type::Thread => write!(f, "tread"),
            Type::Event => write!(f, "event"),
            Type::Mutex => write!(f, "mutex"),
            Type::Mutant => write!(f, "mutant"),
            Type::Semaphore => write!(f, "semaphore"),
            Type::Timer => write!(f, "timer"),
            Type::Desktop => write!(f, "desktop"),
            Type::WindowsStation => write!(f, "windows-station"),
            Type::Winsta => write!(f, "winsta"),
            Type::Key => write!(f, "key"),
            Type::Token => write!(f, "token"),
            Type::Job => write!(f, "job"),
            Type::File => write!(f, "file"),
            Type::Directory => write!(f, "directory"),
            Type::Alpc => write!(f, "alpc"),
            Type::Port => write!(f, "port"),
            Type::ActiveDirectory => write!(f, "active-directory"),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// interpret value as a decimal value
    #[arg(short, long)]
    pub decimal: bool,

    /// Specify a type for the access mask.
    #[arg(short, long, value_enum)]
    pub r#type: Option<Type>,

    /// value is interpreted as hexadecimal, unless the -d switch is specified.
    /// Specific access mask bits will not be interpreted if type is not specified
    pub value: String,
}
