//! ADXL 345 registry mapping
//!
//! Reference: <https://www.analog.com/media/en/technical-documentation/data-sheets/adxl345.pdf>

#![allow(non_camel_case_types)]

use super::reg_helper::*;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

// I2c Addresses
pub const ADXL_ADDR: u8 = 0x53; // ALT Pin Low
pub const ADXL_ADDR_ALT: u8 = 0x1D; // ALT Pin High

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                           ADXL 345 Registers
// —————————————————————————————————————————————————————————————————————————————————————————————————

// Empty Flag Field ID
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FL {}

pub const DEVID: Register<FL> = validated(Register {
    addr:   0x00,
    access: Mode::R,
    bytes:  1,
    order:  End::Little,
    fields: None,
});

// 0x01–0x1C reserved

pub const THRESH_TAP: Register<FL> = validated(Register {
    addr:   0x1D,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});
pub const OFSX: Register<FL> = validated(Register {
    addr:   0x1E,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});
pub const OFSY: Register<FL> = validated(Register {
    addr:   0x1F,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});
pub const OFSZ: Register<FL> = validated(Register {
    addr:   0x20,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});
pub const DUR: Register<FL> = validated(Register {
    addr:   0x21,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});
pub const LATENT: Register<FL> = validated(Register {
    addr:   0x22,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});
pub const WINDOW: Register<FL> = validated(Register {
    addr:   0x23,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});
pub const THRESH_ACT: Register<FL> = validated(Register {
    addr:   0x24,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});
pub const THRESH_INACT: Register<FL> = validated(Register {
    addr:   0x25,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});
pub const TIME_INACT: Register<FL> = validated(Register {
    addr:   0x26,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FL_AIC {
    ACT_ACDC,
    ACT_X_EN,
    ACT_Y_EN,
    ACT_Z_EN,
    INACT_ACDC,
    INACT_X_EN,
    INACT_Y_EN,
    INACT_Z_EN,
}

pub const ACT_INACT_CTL: Register<FL_AIC> = validated(Register {
    addr:   0x27,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_AIC::ACT_ACDC,
            width:  1,
            offset: 7,
        },
        Field {
            id:     FL_AIC::ACT_X_EN,
            width:  1,
            offset: 6,
        },
        Field {
            id:     FL_AIC::ACT_Y_EN,
            width:  1,
            offset: 5,
        },
        Field {
            id:     FL_AIC::ACT_Z_EN,
            width:  1,
            offset: 4,
        },
        Field {
            id:     FL_AIC::INACT_ACDC,
            width:  1,
            offset: 3,
        },
        Field {
            id:     FL_AIC::INACT_X_EN,
            width:  1,
            offset: 2,
        },
        Field {
            id:     FL_AIC::INACT_Y_EN,
            width:  1,
            offset: 1,
        },
        Field {
            id:     FL_AIC::INACT_Z_EN,
            width:  1,
            offset: 0,
        },
    ]),
});

pub const THRESH_FF: Register<FL> = validated(Register {
    addr:   0x28,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});
pub const TIME_FF: Register<FL> = validated(Register {
    addr:   0x29,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: None,
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FL_TA {
    SUPPRESS,
    TAP_X_EN,
    TAP_Y_EN,
    TAP_Z_EN,
}

pub const TAP_AXES: Register<FL_TA> = validated(Register {
    addr:   0x2A,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_TA::SUPPRESS,
            width:  1,
            offset: 3,
        },
        Field {
            id:     FL_TA::TAP_X_EN,
            width:  1,
            offset: 2,
        },
        Field {
            id:     FL_TA::TAP_Y_EN,
            width:  1,
            offset: 1,
        },
        Field {
            id:     FL_TA::TAP_Z_EN,
            width:  1,
            offset: 0,
        },
    ]),
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FL_ATS {
    ACT_X_SRC,
    ACT_Y_SRC,
    ACT_Z_SRC,
    ASLEEP,
    TAP_X_SRC,
    TAP_Y_SRC,
    TAP_Z_SRC,
}

pub const ACT_TAP_STATUS: Register<FL_ATS> = validated(Register {
    addr:   0x2B,
    access: Mode::R,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_ATS::ACT_X_SRC,
            width:  1,
            offset: 6,
        },
        Field {
            id:     FL_ATS::ACT_Y_SRC,
            width:  1,
            offset: 5,
        },
        Field {
            id:     FL_ATS::ACT_Z_SRC,
            width:  1,
            offset: 4,
        },
        Field {
            id:     FL_ATS::ASLEEP,
            width:  1,
            offset: 3,
        },
        Field {
            id:     FL_ATS::TAP_X_SRC,
            width:  1,
            offset: 2,
        },
        Field {
            id:     FL_ATS::TAP_Y_SRC,
            width:  1,
            offset: 1,
        },
        Field {
            id:     FL_ATS::TAP_Z_SRC,
            width:  1,
            offset: 0,
        },
    ]),
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FL_BR {
    LOW_POWER,
    RATE,
}

pub const BW_RATE: Register<FL_BR> = validated(Register {
    addr:   0x2C,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_BR::LOW_POWER,
            width:  1,
            offset: 4,
        },
        Field {
            id:     FL_BR::RATE,
            width:  4,
            offset: 0,
        },
    ]),
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FL_PC {
    LINK,
    AUTO_SLEEP,
    MEASURE,
    SLEEP,
    WAKEUP,
}

pub const POWER_CTL: Register<FL_PC> = validated(Register {
    addr:   0x2D,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_PC::LINK,
            width:  1,
            offset: 5,
        },
        Field {
            id:     FL_PC::AUTO_SLEEP,
            width:  1,
            offset: 4,
        },
        Field {
            id:     FL_PC::MEASURE,
            width:  1,
            offset: 3,
        },
        Field {
            id:     FL_PC::SLEEP,
            width:  1,
            offset: 2,
        },
        Field {
            id:     FL_PC::WAKEUP,
            width:  2,
            offset: 0,
        },
    ]),
});

// Flags used by INT_ENABLE, INT_MAP, INT_SOURCE
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FL_I {
    DATA_READY,
    SINGLE_TAP,
    DOUBLE_TAP,
    ACTIVITY,
    INACTIVITY,
    FREE_FALL,
    WATERMARK,
    OVERRUN,
}

pub const INT_ENABLE: Register<FL_I> = validated(Register {
    addr:   0x2E,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_I::DATA_READY,
            width:  1,
            offset: 7,
        },
        Field {
            id:     FL_I::SINGLE_TAP,
            width:  1,
            offset: 6,
        },
        Field {
            id:     FL_I::DOUBLE_TAP,
            width:  1,
            offset: 5,
        },
        Field {
            id:     FL_I::ACTIVITY,
            width:  1,
            offset: 4,
        },
        Field {
            id:     FL_I::INACTIVITY,
            width:  1,
            offset: 3,
        },
        Field {
            id:     FL_I::FREE_FALL,
            width:  1,
            offset: 2,
        },
        Field {
            id:     FL_I::WATERMARK,
            width:  1,
            offset: 1,
        },
        Field {
            id:     FL_I::OVERRUN,
            width:  1,
            offset: 0,
        },
    ]),
});

pub const INT_MAP: Register<FL_I> = validated(Register {
    addr:   0x2F,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_I::DATA_READY,
            width:  1,
            offset: 7,
        },
        Field {
            id:     FL_I::SINGLE_TAP,
            width:  1,
            offset: 6,
        },
        Field {
            id:     FL_I::DOUBLE_TAP,
            width:  1,
            offset: 5,
        },
        Field {
            id:     FL_I::ACTIVITY,
            width:  1,
            offset: 4,
        },
        Field {
            id:     FL_I::INACTIVITY,
            width:  1,
            offset: 3,
        },
        Field {
            id:     FL_I::FREE_FALL,
            width:  1,
            offset: 2,
        },
        Field {
            id:     FL_I::WATERMARK,
            width:  1,
            offset: 1,
        },
        Field {
            id:     FL_I::OVERRUN,
            width:  1,
            offset: 0,
        },
    ]),
});

pub const INT_SOURCE: Register<FL_I> = validated(Register {
    addr:   0x30,
    access: Mode::R,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_I::DATA_READY,
            width:  1,
            offset: 7,
        },
        Field {
            id:     FL_I::SINGLE_TAP,
            width:  1,
            offset: 6,
        },
        Field {
            id:     FL_I::DOUBLE_TAP,
            width:  1,
            offset: 5,
        },
        Field {
            id:     FL_I::ACTIVITY,
            width:  1,
            offset: 4,
        },
        Field {
            id:     FL_I::INACTIVITY,
            width:  1,
            offset: 3,
        },
        Field {
            id:     FL_I::FREE_FALL,
            width:  1,
            offset: 2,
        },
        Field {
            id:     FL_I::WATERMARK,
            width:  1,
            offset: 1,
        },
        Field {
            id:     FL_I::OVERRUN,
            width:  1,
            offset: 0,
        },
    ]),
});

// FIFO
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FL_DF {
    SELF_TEST,
    SPI,
    INT_INVERT,
    FULL_RES,
    JUSTIFY,
    RANGE,
}

pub const DATA_FORMAT: Register<FL_DF> = validated(Register {
    addr:   0x31,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_DF::SELF_TEST,
            width:  1,
            offset: 7,
        },
        Field {
            id:     FL_DF::SPI,
            width:  1,
            offset: 6,
        },
        Field {
            id:     FL_DF::INT_INVERT,
            width:  1,
            offset: 5,
        },
        Field {
            id:     FL_DF::FULL_RES,
            width:  1,
            offset: 3,
        },
        Field {
            id:     FL_DF::JUSTIFY,
            width:  1,
            offset: 2,
        },
        Field {
            id:     FL_DF::RANGE,
            width:  2,
            offset: 0,
        },
    ]),
});

// Acceleration axis outputs (I16)
pub const DATAX0: Register<FL> = validated(Register {
    addr:   0x32,
    access: Mode::R,
    bytes:  2,
    order:  End::Little,
    fields: None,
});
pub const DATAY0: Register<FL> = validated(Register {
    addr:   0x34,
    access: Mode::R,
    bytes:  2,
    order:  End::Little,
    fields: None,
});
pub const DATAZ0: Register<FL> = validated(Register {
    addr:   0x36,
    access: Mode::R,
    bytes:  2,
    order:  End::Little,
    fields: None,
});

// FIFO
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FL_FC {
    FIFO_MODE,
    TRIGGER,
    SAMPLES,
}

pub const FIFO_CTL: Register<FL_FC> = validated(Register {
    addr:   0x38,
    access: Mode::RW,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_FC::FIFO_MODE,
            width:  2,
            offset: 6,
        },
        Field {
            id:     FL_FC::TRIGGER,
            width:  1,
            offset: 5,
        },
        Field {
            id:     FL_FC::SAMPLES,
            width:  5,
            offset: 0,
        },
    ]),
});

// FIFO_STATUS
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FL_FS {
    FIFO_TRIG,
    ENTRIES,
}

pub const FIFO_STATUS: Register<FL_FS> = validated(Register {
    addr:   0x39,
    access: Mode::R,
    bytes:  1,
    order:  End::Little,
    fields: Some(&[
        Field {
            id:     FL_FS::FIFO_TRIG,
            width:  1,
            offset: 7,
        },
        Field {
            id:     FL_FS::ENTRIES,
            width:  6,
            offset: 0,
        },
    ]),
});
