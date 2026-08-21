//! The [`AcpiError`] error type

// Adapted from source/include/acexcep.h

use core::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(crate) struct AcpiStatus(u32);

/// An error which ACPICA produced, or which can be given to ACPICA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[non_exhaustive]
pub enum AcpiError {
    /*
     * Environmental exceptions
     */
    /// Unspecified error
    Error = 0x0001 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// ACPI tables could not be found
    NoAcpiTables = 0x0002 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// A namespace has not been loaded
    NoNamespace = 0x0003 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Insufficient dynamic memory
    NoMemory = 0x0004 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// A requested entity is not found
    NotFound = 0x0005 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// A required entity does not exist
    NotExist = 0x0006 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// An entity already exists
    AlreadyExists = 0x0007 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// The object type is incorrect
    Type = 0x0008 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// A required object was missing
    NullObject = 0x0009 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// The requested object does not exist
    NullEntry = 0x000A | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// The buffer provided is too small
    BufferOverflow = 0x000B | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// An internal stack overflowed
    StackOverflow = 0x000C | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// An internal stack underflowed
    StackUnderflow = 0x000D | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// The feature is not implemented
    NotImplemented = 0x000E | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// The feature is not supported
    Support = 0x000F | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// A predefined limit was exceeded
    Limit = 0x0010 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// A time limit or timeout expired
    Time = 0x0011 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Internal error, attempt was made to acquire a mutex in improper order
    AcquireDeadlock = 0x0012 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Internal error, attempt was made to release a mutex in improper order
    ReleaseDeadlock = 0x0013 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// An attempt to release a mutex or Global Lock without a previous acquire
    NotAcquired = 0x0014 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Internal error, attempt was made to acquire a mutex twice
    AlreadyAcquired = 0x0015 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Hardware did not respond after an I/O operation
    NoHardwareResponse = 0x0016 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// There is no FACS Global Lock
    NoGlobalLock = 0x0017 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// A control method was aborted
    AbortMethod = 0x0018 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Attempt was made to install the same handler that is already installed
    SameHandler = 0x0019 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// A handler for the operation is not installed
    NoHandler = 0x001A | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// There are no more Owner IDs available for ACPI tables or control methods
    OwnerIdLimit = 0x001B | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// The interface is not part of the current subsystem configuration
    NotConfigured = 0x001C | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Permission denied for the requested operation
    Access = 0x001D | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// An I/O error occurred
    IoError = 0x001E | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Overflow during string-to-integer conversion
    NumericOverflow = 0x001F | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Overflow during ASCII hex-to-binary conversion
    HexOverflow = 0x0020 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Overflow during ASCII decimal-to-binary conversion
    DecimalOverflow = 0x0021 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Overflow during ASCII octal-to-binary conversion
    OctalOverflow = 0x0022 | AcpiError::AE_CODE_ENVIRONMENTAL,
    /// Reached the end of table
    EndOfTable = 0x0023 | AcpiError::AE_CODE_ENVIRONMENTAL,

    /*
     * Programmer exceptions
     */
    /// A parameter is out of range or invalid
    BadParameter = 0x0001 | AcpiError::AE_CODE_PROGRAMMER,
    /// An invalid character was found in a name
    BadCharacter = 0x0002 | AcpiError::AE_CODE_PROGRAMMER,
    /// An invalid character was found in a pathname
    BadPathname = 0x0003 | AcpiError::AE_CODE_PROGRAMMER,
    /// A package or buffer contained incorrect data
    BadData = 0x0004 | AcpiError::AE_CODE_PROGRAMMER,
    /// Invalid character in a Hex constant
    BadHexConstant = 0x0005 | AcpiError::AE_CODE_PROGRAMMER,
    /// Invalid character in an Octal constant
    BadOctalConstant = 0x0006 | AcpiError::AE_CODE_PROGRAMMER,
    /// Invalid character in a Decimal constant
    BadDecimalConstant = 0x0007 | AcpiError::AE_CODE_PROGRAMMER,
    /// Too few arguments were passed to a control method
    MissingArguments = 0x0008 | AcpiError::AE_CODE_PROGRAMMER,
    /// An illegal null I/O address
    BadAddress = 0x0009 | AcpiError::AE_CODE_PROGRAMMER,

    /*
     * Acpi table exceptions
     */
    /// An ACPI table has an invalid signature
    BadSignature = 0x0001 | AcpiError::AE_CODE_ACPI_TABLES,
    /// Invalid field in an ACPI table header
    BadHeader = 0x0002 | AcpiError::AE_CODE_ACPI_TABLES,
    /// An ACPI table checksum is not correct
    BadChecksum = 0x0003 | AcpiError::AE_CODE_ACPI_TABLES,
    /// An invalid value was found in a table
    BadValue = 0x0004 | AcpiError::AE_CODE_ACPI_TABLES,
    /// The FADT or FACS has improper length
    InvalidTableLength = 0x0005 | AcpiError::AE_CODE_ACPI_TABLES,

    /*
     * AML exceptions. These are caused by problems with
     * the actual AML byte stream
     */
    /// Invalid AML opcode encountered
    AmlBadOpcode = 0x0001 | AcpiError::AE_CODE_AML,
    /// A required operand is missing
    AmlNoOperand = 0x0002 | AcpiError::AE_CODE_AML,
    /// An operand of an incorrect type was encountered
    AmlOperandType = 0x0003 | AcpiError::AE_CODE_AML,
    /// The operand had an inappropriate or invalid value
    AmlOperandValue = 0x0004 | AcpiError::AE_CODE_AML,
    /// Method tried to use an uninitialized local variable
    AmlUninitializedLocal = 0x0005 | AcpiError::AE_CODE_AML,
    /// Method tried to use an uninitialized argument
    AmlUninitializedArg = 0x0006 | AcpiError::AE_CODE_AML,
    /// Method tried to use an empty package element
    AmlUninitializedElement = 0x0007 | AcpiError::AE_CODE_AML,
    /// Overflow during BCD conversion or other
    AmlNumericOverflow = 0x0008 | AcpiError::AE_CODE_AML,
    /// Tried to access beyond the end of an Operation Region
    AmlRegionLimit = 0x0009 | AcpiError::AE_CODE_AML,
    /// Tried to access beyond the end of a buffer
    AmlBufferLimit = 0x000A | AcpiError::AE_CODE_AML,
    /// Tried to access beyond the end of a package
    AmlPackageLimit = 0x000B | AcpiError::AE_CODE_AML,
    /// During execution of AML Divide operator
    AmlDivideByZero = 0x000C | AcpiError::AE_CODE_AML,
    /// An ACPI name contains invalid character(s)
    AmlBadName = 0x000D | AcpiError::AE_CODE_AML,
    /// Could not resolve a named reference
    AmlNameNotFound = 0x000E | AcpiError::AE_CODE_AML,
    /// An internal error within the interpreter
    AmlInternal = 0x000F | AcpiError::AE_CODE_AML,
    /// An Operation Region SpaceID is invalid
    AmlInvalidSpaceId = 0x0010 | AcpiError::AE_CODE_AML,
    /// String is longer than 200 characters
    AmlStringLimit = 0x0011 | AcpiError::AE_CODE_AML,
    /// A method did not return a required value
    AmlNoReturnValue = 0x0012 | AcpiError::AE_CODE_AML,
    /// A control method reached the maximum reentrancy limit of 255
    AmlMethodLimit = 0x0013 | AcpiError::AE_CODE_AML,
    /// A thread tried to release a mutex that it does not own
    AmlNotOwner = 0x0014 | AcpiError::AE_CODE_AML,
    /// Mutex SyncLevel release mismatch
    AmlMutexOrder = 0x0015 | AcpiError::AE_CODE_AML,
    /// Attempt to release a mutex that was not previously acquired
    AmlMutexNotAcquired = 0x0016 | AcpiError::AE_CODE_AML,
    /// Invalid resource type in resource list
    AmlInvalidResourceType = 0x0017 | AcpiError::AE_CODE_AML,
    /// Invalid Argx or Localx (x too large)
    AmlInvalidIndex = 0x0018 | AcpiError::AE_CODE_AML,
    /// Bank value or Index value beyond range of register
    AmlRegisterLimit = 0x0019 | AcpiError::AE_CODE_AML,
    /// Break or Continue without a While
    AmlNoWhile = 0x001A | AcpiError::AE_CODE_AML,
    /// Non-aligned memory transfer on platform that does not support this
    AmlAlignment = 0x001B | AcpiError::AE_CODE_AML,
    /// No End Tag in a resource list
    AmlNoResourceEndTag = 0x001C | AcpiError::AE_CODE_AML,
    /// Invalid value of a resource element
    AmlBadResourceValue = 0x001D | AcpiError::AE_CODE_AML,
    /// Two references refer to each other
    AmlCircularReference = 0x001E | AcpiError::AE_CODE_AML,
    /// The length of a Resource Descriptor in the AML is incorrect
    AmlBadResourceLength = 0x001F | AcpiError::AE_CODE_AML,
    /// A memory, I/O, or PCI configuration address is invalid
    AmlIllegalAddress = 0x0020 | AcpiError::AE_CODE_AML,
    /// An AML While loop exceeded the maximum execution time
    AmlLoopTimeout = 0x0021 | AcpiError::AE_CODE_AML,
    /// A namespace node is uninitialized or unresolved
    AmlUninitializedNode = 0x0022 | AcpiError::AE_CODE_AML,
    /// A target operand of an incorrect type was encountered
    AmlTargetType = 0x0023 | AcpiError::AE_CODE_AML,
    /// Violation of a fixed ACPI protocol
    AmlProtocol = 0x0024 | AcpiError::AE_CODE_AML,
    /// The length of the buffer is invalid/incorrect
    AmlBufferLength = 0x0025 | AcpiError::AE_CODE_AML,

    /*
     * Internal exceptions used for control
     */
    /// A Method returned a value
    ControlReturnValue = 0x0001 | AcpiError::AE_CODE_CONTROL,
    /// Method is calling another method
    ControlPending = 0x0002 | AcpiError::AE_CODE_CONTROL,
    /// Terminate the executing method
    ControlTerminate = 0x0003 | AcpiError::AE_CODE_CONTROL,
    /// An If or While predicate result
    ControlTrue = 0x0004 | AcpiError::AE_CODE_CONTROL,
    /// An If or While predicate result
    ControlFalse = 0x0005 | AcpiError::AE_CODE_CONTROL,
    /// Maximum search depth has been reached
    ControlDepth = 0x0006 | AcpiError::AE_CODE_CONTROL,
    /// An If or While predicate is false
    ControlEnd = 0x0007 | AcpiError::AE_CODE_CONTROL,
    /// Transfer control to called method
    ControlTransfer = 0x0008 | AcpiError::AE_CODE_CONTROL,
    /// A Break has been executed
    ControlBreak = 0x0009 | AcpiError::AE_CODE_CONTROL,
    /// A Continue has been executed
    ControlContinue = 0x000A | AcpiError::AE_CODE_CONTROL,
    /// Used to skip over bad opcodes
    ControlParseContinue = 0x000B | AcpiError::AE_CODE_CONTROL,
    /// Used to implement AML While loops
    ControlParsePending = 0x000C | AcpiError::AE_CODE_CONTROL,

    /// Unknown error
    Unknown,
}

impl AcpiError {
    const AE_CODE_ENVIRONMENTAL: u32 = 0x0000;
    const AE_CODE_PROGRAMMER: u32 = 0x1000;
    const AE_CODE_ACPI_TABLES: u32 = 0x2000;
    const AE_CODE_AML: u32 = 0x3000;
    const AE_CODE_CONTROL: u32 = 0x4000;
    const AE_CODE_MASK: u32 = 0xF000;
}

/// A general type of [`AcpiError`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AcpiErrorClass {
    /// The error was either internal to ACPICA or was caused by an error in the execution environment or hardware
    Environmental,
    /// The error was caused by a software bug
    Programmer,
    /// An ACPI table was invalid
    AcpiTables,
    /// An error occurred while parsing or executing AML code
    Aml,
    /// Internal errors used for control flow within ACPICA
    Control,
    /// The error type was unknown to the rust bindings to ACPICA
    Unknown,
}

impl AcpiError {
    /// Gets the general type of error
    #[must_use]
    pub const fn class(&self) -> AcpiErrorClass {
        if let Self::Unknown = self {
            return AcpiErrorClass::Unknown;
        }

        match *self as u32 & Self::AE_CODE_MASK {
            _e @ Self::AE_CODE_ENVIRONMENTAL => AcpiErrorClass::Environmental,
            _e @ Self::AE_CODE_PROGRAMMER => AcpiErrorClass::Programmer,
            _e @ Self::AE_CODE_ACPI_TABLES => AcpiErrorClass::AcpiTables,
            _e @ Self::AE_CODE_AML => AcpiErrorClass::Aml,
            _e @ Self::AE_CODE_CONTROL => AcpiErrorClass::Control,
            _ => AcpiErrorClass::Unknown,
        }
    }
}

pub(crate) trait AcpiErrorAsStatusExt {
    fn to_acpi_status(&self) -> AcpiStatus;
}

impl AcpiErrorAsStatusExt for Result<(), AcpiError> {
    fn to_acpi_status(&self) -> AcpiStatus {
        match self {
            Ok(()) => AcpiStatus(0),
            Err(e) => e.to_acpi_status(),
        }
    }
}

impl AcpiErrorAsStatusExt for AcpiError {
    fn to_acpi_status(&self) -> AcpiStatus {
        if let AcpiError::Unknown = self {
            AcpiStatus(Self::Error as u32)
        } else {
            AcpiStatus(*self as u32)
        }
    }
}

impl AcpiStatus {
    pub const OK: Self = Self(0);

    #[allow(clippy::too_many_lines)]
    pub fn as_result(self) -> Result<(), AcpiError> {
        match self.0 {
            0 => Ok(()),
            e if e == 0x0001 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::Error),
            e if e == 0x0002 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NoAcpiTables),
            e if e == 0x0003 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NoNamespace),
            e if e == 0x0004 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NoMemory),
            e if e == 0x0005 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NotFound),
            e if e == 0x0006 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NotExist),
            e if e == 0x0007 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::AlreadyExists),
            e if e == 0x0008 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::Type),
            e if e == 0x0009 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NullObject),
            e if e == 0x000A | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NullEntry),
            e if e == 0x000B | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::BufferOverflow),
            e if e == 0x000C | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::StackOverflow),
            e if e == 0x000D | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::StackUnderflow),
            e if e == 0x000E | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NotImplemented),
            e if e == 0x000F | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::Support),
            e if e == 0x0010 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::Limit),
            e if e == 0x0011 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::Time),
            e if e == 0x0012 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::AcquireDeadlock),
            e if e == 0x0013 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::ReleaseDeadlock),
            e if e == 0x0014 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NotAcquired),
            e if e == 0x0015 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::AlreadyAcquired),
            e if e == 0x0016 | AcpiError::AE_CODE_ENVIRONMENTAL => {
                Err(AcpiError::NoHardwareResponse)
            }
            e if e == 0x0017 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NoGlobalLock),
            e if e == 0x0018 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::AbortMethod),
            e if e == 0x0019 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::SameHandler),
            e if e == 0x001A | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NoHandler),
            e if e == 0x001B | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::OwnerIdLimit),
            e if e == 0x001C | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NotConfigured),
            e if e == 0x001D | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::Access),
            e if e == 0x001E | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::IoError),
            e if e == 0x001F | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::NumericOverflow),
            e if e == 0x0020 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::HexOverflow),
            e if e == 0x0021 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::DecimalOverflow),
            e if e == 0x0022 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::OctalOverflow),
            e if e == 0x0023 | AcpiError::AE_CODE_ENVIRONMENTAL => Err(AcpiError::EndOfTable),
            e if e == 0x0001 | AcpiError::AE_CODE_PROGRAMMER => Err(AcpiError::BadParameter),
            e if e == 0x0002 | AcpiError::AE_CODE_PROGRAMMER => Err(AcpiError::BadCharacter),
            e if e == 0x0003 | AcpiError::AE_CODE_PROGRAMMER => Err(AcpiError::BadPathname),
            e if e == 0x0004 | AcpiError::AE_CODE_PROGRAMMER => Err(AcpiError::BadData),
            e if e == 0x0005 | AcpiError::AE_CODE_PROGRAMMER => Err(AcpiError::BadHexConstant),
            e if e == 0x0006 | AcpiError::AE_CODE_PROGRAMMER => Err(AcpiError::BadOctalConstant),
            e if e == 0x0007 | AcpiError::AE_CODE_PROGRAMMER => Err(AcpiError::BadDecimalConstant),
            e if e == 0x0008 | AcpiError::AE_CODE_PROGRAMMER => Err(AcpiError::MissingArguments),
            e if e == 0x0009 | AcpiError::AE_CODE_PROGRAMMER => Err(AcpiError::BadAddress),
            e if e == 0x0001 | AcpiError::AE_CODE_ACPI_TABLES => Err(AcpiError::BadSignature),
            e if e == 0x0002 | AcpiError::AE_CODE_ACPI_TABLES => Err(AcpiError::BadHeader),
            e if e == 0x0003 | AcpiError::AE_CODE_ACPI_TABLES => Err(AcpiError::BadChecksum),
            e if e == 0x0004 | AcpiError::AE_CODE_ACPI_TABLES => Err(AcpiError::BadValue),
            e if e == 0x0005 | AcpiError::AE_CODE_ACPI_TABLES => Err(AcpiError::InvalidTableLength),
            e if e == 0x0001 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlBadOpcode),
            e if e == 0x0002 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlNoOperand),
            e if e == 0x0003 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlOperandType),
            e if e == 0x0004 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlOperandValue),
            e if e == 0x0005 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlUninitializedLocal),
            e if e == 0x0006 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlUninitializedArg),
            e if e == 0x0007 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlUninitializedElement),
            e if e == 0x0008 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlNumericOverflow),
            e if e == 0x0009 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlRegionLimit),
            e if e == 0x000A | AcpiError::AE_CODE_AML => Err(AcpiError::AmlBufferLimit),
            e if e == 0x000B | AcpiError::AE_CODE_AML => Err(AcpiError::AmlPackageLimit),
            e if e == 0x000C | AcpiError::AE_CODE_AML => Err(AcpiError::AmlDivideByZero),
            e if e == 0x000D | AcpiError::AE_CODE_AML => Err(AcpiError::AmlBadName),
            e if e == 0x000E | AcpiError::AE_CODE_AML => Err(AcpiError::AmlNameNotFound),
            e if e == 0x000F | AcpiError::AE_CODE_AML => Err(AcpiError::AmlInternal),
            e if e == 0x0010 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlInvalidSpaceId),
            e if e == 0x0011 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlStringLimit),
            e if e == 0x0012 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlNoReturnValue),
            e if e == 0x0013 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlMethodLimit),
            e if e == 0x0014 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlNotOwner),
            e if e == 0x0015 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlMutexOrder),
            e if e == 0x0016 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlMutexNotAcquired),
            e if e == 0x0017 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlInvalidResourceType),
            e if e == 0x0018 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlInvalidIndex),
            e if e == 0x0019 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlRegisterLimit),
            e if e == 0x001A | AcpiError::AE_CODE_AML => Err(AcpiError::AmlNoWhile),
            e if e == 0x001B | AcpiError::AE_CODE_AML => Err(AcpiError::AmlAlignment),
            e if e == 0x001C | AcpiError::AE_CODE_AML => Err(AcpiError::AmlNoResourceEndTag),
            e if e == 0x001D | AcpiError::AE_CODE_AML => Err(AcpiError::AmlBadResourceValue),
            e if e == 0x001E | AcpiError::AE_CODE_AML => Err(AcpiError::AmlCircularReference),
            e if e == 0x001F | AcpiError::AE_CODE_AML => Err(AcpiError::AmlBadResourceLength),
            e if e == 0x0020 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlIllegalAddress),
            e if e == 0x0021 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlLoopTimeout),
            e if e == 0x0022 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlUninitializedNode),
            e if e == 0x0023 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlTargetType),
            e if e == 0x0024 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlProtocol),
            e if e == 0x0025 | AcpiError::AE_CODE_AML => Err(AcpiError::AmlBufferLength),
            e if e == 0x0001 | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlReturnValue),
            e if e == 0x0002 | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlPending),
            e if e == 0x0003 | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlTerminate),
            e if e == 0x0004 | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlTrue),
            e if e == 0x0005 | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlFalse),
            e if e == 0x0006 | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlDepth),
            e if e == 0x0007 | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlEnd),
            e if e == 0x0008 | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlTransfer),
            e if e == 0x0009 | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlBreak),
            e if e == 0x000A | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlContinue),
            e if e == 0x000B | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlParseContinue),
            e if e == 0x000C | AcpiError::AE_CODE_CONTROL => Err(AcpiError::ControlParsePending),

            _ => Err(AcpiError::Unknown),
        }
    }
}

impl Display for AcpiError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let err_string = match self {
            Self::Error => "Unspecified error",
            Self::NoAcpiTables => "ACPI tables could not be found",
            Self::NoNamespace => "A namespace has not been loaded",
            Self::NoMemory => "Insufficient dynamic memory",
            Self::NotFound => "A requested entity is not found",
            Self::NotExist => "A required entity does not exist",
            Self::AlreadyExists => "An entity already exists",
            Self::Type => "The object type is incorrect",
            Self::NullObject => "A required object was missing",
            Self::NullEntry => "The requested object does not exist",
            Self::BufferOverflow => "The buffer provided is too small",
            Self::StackOverflow => "An internal stack overflowed",
            Self::StackUnderflow => "An internal stack underflowed",
            Self::NotImplemented => "The feature is not implemented",
            Self::Support => "The feature is not supported",
            Self::Limit => "A predefined limit was exceeded",
            Self::Time => "A time limit or timeout expired",
            Self::AcquireDeadlock => {
                "Internal error, attempt was made to acquire a mutex in improper order"
            }
            Self::ReleaseDeadlock => {
                "Internal error, attempt was made to release a mutex in improper order"
            }
            Self::NotAcquired => {
                "An attempt to release a mutex or Global Lock without a previous acquire"
            }
            Self::AlreadyAcquired => "Internal error, attempt was made to acquire a mutex twice",
            Self::NoHardwareResponse => "Hardware did not respond after an I/O operation",
            Self::NoGlobalLock => "There is no FACS Global Lock",
            Self::AbortMethod => "A control method was aborted",
            Self::SameHandler => {
                "Attempt was made to install the same handler that is already installed"
            }
            Self::NoHandler => "A handler for the operation is not installed",
            Self::OwnerIdLimit => {
                "There are no more Owner IDs available for ACPI tables or control methods"
            }
            Self::NotConfigured => {
                "The interface is not part of the current subsystem configuration"
            }
            Self::Access => "Permission denied for the requested operation",
            Self::IoError => "An I/O error occurred",
            Self::NumericOverflow => "Overflow during string-to-integer conversion",
            Self::HexOverflow => "Overflow during ASCII hex-to-binary conversion",
            Self::DecimalOverflow => "Overflow during ASCII decimal-to-binary conversion",
            Self::OctalOverflow => "Overflow during ASCII octal-to-binary conversion",
            Self::EndOfTable => "Reached the end of table",
            Self::BadParameter => "A parameter is out of range or invalid",
            Self::BadCharacter => "An invalid character was found in a name",
            Self::BadPathname => "An invalid character was found in a pathname",
            Self::BadData => "A package or buffer contained incorrect data",
            Self::BadHexConstant => "Invalid character in a Hex constant",
            Self::BadOctalConstant => "Invalid character in an Octal constant",
            Self::BadDecimalConstant => "Invalid character in a Decimal constant",
            Self::MissingArguments => "Too few arguments were passed to a control method",
            Self::BadAddress => "An illegal null I/O address",
            Self::BadSignature => "An ACPI table has an invalid signature",
            Self::BadHeader => "Invalid field in an ACPI table header",
            Self::BadChecksum => "An ACPI table checksum is not correct",
            Self::BadValue => "An invalid value was found in a table",
            Self::InvalidTableLength => "The FADT or FACS has improper length",
            Self::AmlBadOpcode => "Invalid AML opcode encountered",
            Self::AmlNoOperand => "A required operand is missing",
            Self::AmlOperandType => "An operand of an incorrect type was encountered",
            Self::AmlOperandValue => "The operand had an inappropriate or invalid value",
            Self::AmlUninitializedLocal => "Method tried to use an uninitialized local variable",
            Self::AmlUninitializedArg => "Method tried to use an uninitialized argument",
            Self::AmlUninitializedElement => "Method tried to use an empty package element",
            Self::AmlNumericOverflow => "Overflow during BCD conversion or other",
            Self::AmlRegionLimit => "Tried to access beyond the end of an Operation Region",
            Self::AmlBufferLimit => "Tried to access beyond the end of a buffer",
            Self::AmlPackageLimit => "Tried to access beyond the end of a package",
            Self::AmlDivideByZero => "During execution of AML Divide operator",
            Self::AmlBadName => "An ACPI name contains invalid character(s)",
            Self::AmlNameNotFound => "Could not resolve a named reference",
            Self::AmlInternal => "An internal error within the interpreter",
            Self::AmlInvalidSpaceId => "An Operation Region SpaceID is invalid",
            Self::AmlStringLimit => "String is longer than 200 characters",
            Self::AmlNoReturnValue => "A method did not return a required value",
            Self::AmlMethodLimit => "A control method reached the maximum reentrancy limit of 255",
            Self::AmlNotOwner => "A thread tried to release a mutex that it does not own",
            Self::AmlMutexOrder => "Mutex SyncLevel release mismatch",
            Self::AmlMutexNotAcquired => {
                "Attempt to release a mutex that was not previously acquired"
            }
            Self::AmlInvalidResourceType => "Invalid resource type in resource list",
            Self::AmlInvalidIndex => "Invalid Argx or Localx (x too large)",
            Self::AmlRegisterLimit => "Bank value or Index value beyond range of register",
            Self::AmlNoWhile => "Break or Continue without a While",
            Self::AmlAlignment => {
                "Non-aligned memory transfer on platform that does not support this"
            }
            Self::AmlNoResourceEndTag => "No End Tag in a resource list",
            Self::AmlBadResourceValue => "Invalid value of a resource element",
            Self::AmlCircularReference => "Two references refer to each other",
            Self::AmlBadResourceLength => {
                "The length of a Resource Descriptor in the AML is incorrect"
            }
            Self::AmlIllegalAddress => "A memory, I/O, or PCI configuration address is invalid",
            Self::AmlLoopTimeout => "An AML While loop exceeded the maximum execution time",
            Self::AmlUninitializedNode => "A namespace node is uninitialized or unresolved",
            Self::AmlTargetType => "A target operand of an incorrect type was encountered",
            Self::AmlProtocol => "Violation of a fixed ACPI protocol",
            Self::AmlBufferLength => "The length of the buffer is invalid/incorrect",
            Self::ControlReturnValue => "A Method returned a value",
            Self::ControlPending => "Method is calling another method",
            Self::ControlTerminate => "Terminate the executing method",
            Self::ControlTrue | Self::ControlFalse => "An If or While predicate result",
            Self::ControlDepth => "Maximum search depth has been reached",
            Self::ControlEnd => "An If or While predicate is false",
            Self::ControlTransfer => "Transfer control to called method",
            Self::ControlBreak => "A Break has been executed",
            Self::ControlContinue => "A Continue has been executed",
            Self::ControlParseContinue => "Used to skip over bad opcodes",
            Self::ControlParsePending => "Used to implement AML While loops",

            Self::Unknown => "Unknown error",
        };

        f.write_str(err_string)
    }
}

/// Tests that [`as_result`][AcpiStatus::as_result] is the inverse of [`as_acpi_status`][AcpiError::as_acpi_status]
#[test]
fn test_as_result() {
    assert_eq!(AcpiStatus(0).as_result(), Ok(()));

    for i in 1..0x4f00 {
        let error = AcpiStatus(i)
            .as_result()
            .expect_err("An AcpiStatus with a non-0 value should not map to Ok");

        if error == AcpiError::Unknown {
            continue;
        }

        assert_eq!(error.to_acpi_status().0, i);
    }
}

/// Tests that [`class`][`AcpiError::class`] returns correct values
#[test]
fn test_error_class() {
    assert_eq!(AcpiError::Error.class(), AcpiErrorClass::Environmental);
    assert_eq!(
        AcpiError::StackOverflow.class(),
        AcpiErrorClass::Environmental
    );
    assert_eq!(AcpiError::BadParameter.class(), AcpiErrorClass::Programmer);
    assert_eq!(
        AcpiError::MissingArguments.class(),
        AcpiErrorClass::Programmer
    );
    assert_eq!(AcpiError::BadSignature.class(), AcpiErrorClass::AcpiTables);
    assert_eq!(AcpiError::BadChecksum.class(), AcpiErrorClass::AcpiTables);
    assert_eq!(AcpiError::AmlBadOpcode.class(), AcpiErrorClass::Aml);
    assert_eq!(AcpiError::AmlInternal.class(), AcpiErrorClass::Aml);
    assert_eq!(AcpiError::ControlPending.class(), AcpiErrorClass::Control);
    assert_eq!(AcpiError::ControlContinue.class(), AcpiErrorClass::Control);

    assert_eq!(AcpiError::Unknown.class(), AcpiErrorClass::Unknown);
}
