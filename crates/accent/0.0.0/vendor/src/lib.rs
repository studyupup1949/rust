#![allow(unused)]

pub mod color;
pub mod model;
pub mod space;

pub mod family {
    pub mod rgb {
        mod wg;
        mod sc;
        mod dp3;
        mod srgb;
        mod aces;
        mod rec7;
        mod rec20;
        mod linear;
        mod prophoto;

        pub use prophoto::PROPHOTO;
        pub use linear::LINEAR;
        pub use rec20::REC20;
        pub use rec7::REC7;
        pub use aces::ACES;
        pub use srgb::SRGB;
        pub use dp3::DP3;
        pub use sc::SC;
        pub use wg::WG;
    }

    pub mod hsl {
        mod hsl;
        mod hsv;
        mod hsb;
        mod hsi;
        mod hwb;
        mod hcg;

        pub use hsl::HSL;
        pub use hsv::HSV;
        pub use hsb::HSB;
        pub use hsi::HSI;
        pub use hwb::HWB;
        pub use hcg::HCG;
    }

    pub mod lab {
        mod oklab;
        mod oklch;
        mod lchuv;
        mod hlab;
        mod lab;
        mod lch;
        mod luv;

        pub use oklab::OKLAB;
        pub use oklch::OKLCH;
        pub use lchuv::LCHUV;
        pub use hlab::HLAB;
        pub use lab::LAB;
        pub use lch::LCH;
        pub use luv::LUV;
    }

    pub mod cmyk {
        mod device;
        mod uswcc;
        mod fog39;
        mod fog51;
        mod swop;
        mod jcc;
        mod gcc;

        pub use device::DEVICE;
        pub use uswcc::USWCC;
        pub use fog39::FOG39;
        pub use fog51::FOG51;
        pub use swop::SWOP;
        pub use jcc::JCC;
        pub use gcc::GCC;
    }

    pub mod extra {
        mod hex;
        mod xyz;

        pub use hex::HEX;
        pub use xyz::XYZ;
    }
}
