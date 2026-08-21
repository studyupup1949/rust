#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Port Configuration"]
    pub cfg: CFG,
    #[doc = "0x04 - Port Output Enable"]
    pub oen: OEN,
    _reserved2: [u8; 2usize],
    #[doc = "0x08 - Port Output Pull-up/Pull-down Enable"]
    pub pe: PE,
    _reserved3: [u8; 2usize],
    #[doc = "0x0c - Port Input Path Enable"]
    pub ien: IEN,
    _reserved4: [u8; 2usize],
    #[doc = "0x10 - Port Registered Data Input"]
    pub in_: IN,
    _reserved5: [u8; 2usize],
    #[doc = "0x14 - Port Data Output"]
    pub out: OUT,
    _reserved6: [u8; 2usize],
    #[doc = "0x18 - Port Data Out Set"]
    pub set: SET,
    _reserved7: [u8; 2usize],
    #[doc = "0x1c - Port Data Out Clear"]
    pub clr: CLR,
    _reserved8: [u8; 2usize],
    #[doc = "0x20 - Port Pin Toggle"]
    pub tgl: TGL,
    _reserved9: [u8; 2usize],
    #[doc = "0x24 - Port Interrupt Polarity"]
    pub pol: POL,
    _reserved10: [u8; 2usize],
    #[doc = "0x28 - Port Interrupt A Enable"]
    pub iena: IENA,
    _reserved11: [u8; 2usize],
    #[doc = "0x2c - Port Interrupt B Enable"]
    pub ienb: IENB,
    _reserved12: [u8; 2usize],
    #[doc = "0x30 - Port Interrupt Status"]
    pub int: INT,
    _reserved13: [u8; 2usize],
    #[doc = "0x34 - Port Drive Strength Select"]
    pub ds: DS,
}
#[doc = "Port Configuration\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub type CFG = crate::Reg<u32, _CFG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CFG;
#[doc = "`read()` method returns [cfg::R](cfg::R) reader structure"]
impl crate::Readable for CFG {}
#[doc = "`write(|w| ..)` method takes [cfg::W](cfg::W) writer structure"]
impl crate::Writable for CFG {}
#[doc = "Port Configuration"]
pub mod cfg;
#[doc = "Port Output Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [oen](oen) module"]
pub type OEN = crate::Reg<u16, _OEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _OEN;
#[doc = "`read()` method returns [oen::R](oen::R) reader structure"]
impl crate::Readable for OEN {}
#[doc = "`write(|w| ..)` method takes [oen::W](oen::W) writer structure"]
impl crate::Writable for OEN {}
#[doc = "Port Output Enable"]
pub mod oen;
#[doc = "Port Output Pull-up/Pull-down Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pe](pe) module"]
pub type PE = crate::Reg<u16, _PE>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PE;
#[doc = "`read()` method returns [pe::R](pe::R) reader structure"]
impl crate::Readable for PE {}
#[doc = "`write(|w| ..)` method takes [pe::W](pe::W) writer structure"]
impl crate::Writable for PE {}
#[doc = "Port Output Pull-up/Pull-down Enable"]
pub mod pe;
#[doc = "Port Input Path Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ien](ien) module"]
pub type IEN = crate::Reg<u16, _IEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IEN;
#[doc = "`read()` method returns [ien::R](ien::R) reader structure"]
impl crate::Readable for IEN {}
#[doc = "`write(|w| ..)` method takes [ien::W](ien::W) writer structure"]
impl crate::Writable for IEN {}
#[doc = "Port Input Path Enable"]
pub mod ien;
#[doc = "Port Registered Data Input\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [in_](in_) module"]
pub type IN = crate::Reg<u16, _IN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IN;
#[doc = "`read()` method returns [in_::R](in_::R) reader structure"]
impl crate::Readable for IN {}
#[doc = "Port Registered Data Input"]
pub mod in_;
#[doc = "Port Data Output\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [out](out) module"]
pub type OUT = crate::Reg<u16, _OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _OUT;
#[doc = "`read()` method returns [out::R](out::R) reader structure"]
impl crate::Readable for OUT {}
#[doc = "`write(|w| ..)` method takes [out::W](out::W) writer structure"]
impl crate::Writable for OUT {}
#[doc = "Port Data Output"]
pub mod out;
#[doc = "Port Data Out Set\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [set](set) module"]
pub type SET = crate::Reg<u16, _SET>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SET;
#[doc = "`write(|w| ..)` method takes [set::W](set::W) writer structure"]
impl crate::Writable for SET {}
#[doc = "Port Data Out Set"]
pub mod set;
#[doc = "Port Data Out Clear\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [clr](clr) module"]
pub type CLR = crate::Reg<u16, _CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CLR;
#[doc = "`write(|w| ..)` method takes [clr::W](clr::W) writer structure"]
impl crate::Writable for CLR {}
#[doc = "Port Data Out Clear"]
pub mod clr;
#[doc = "Port Pin Toggle\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tgl](tgl) module"]
pub type TGL = crate::Reg<u16, _TGL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TGL;
#[doc = "`write(|w| ..)` method takes [tgl::W](tgl::W) writer structure"]
impl crate::Writable for TGL {}
#[doc = "Port Pin Toggle"]
pub mod tgl;
#[doc = "Port Interrupt Polarity\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pol](pol) module"]
pub type POL = crate::Reg<u16, _POL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _POL;
#[doc = "`read()` method returns [pol::R](pol::R) reader structure"]
impl crate::Readable for POL {}
#[doc = "`write(|w| ..)` method takes [pol::W](pol::W) writer structure"]
impl crate::Writable for POL {}
#[doc = "Port Interrupt Polarity"]
pub mod pol;
#[doc = "Port Interrupt A Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [iena](iena) module"]
pub type IENA = crate::Reg<u16, _IENA>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IENA;
#[doc = "`read()` method returns [iena::R](iena::R) reader structure"]
impl crate::Readable for IENA {}
#[doc = "`write(|w| ..)` method takes [iena::W](iena::W) writer structure"]
impl crate::Writable for IENA {}
#[doc = "Port Interrupt A Enable"]
pub mod iena;
#[doc = "Port Interrupt B Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ienb](ienb) module"]
pub type IENB = crate::Reg<u16, _IENB>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IENB;
#[doc = "`read()` method returns [ienb::R](ienb::R) reader structure"]
impl crate::Readable for IENB {}
#[doc = "`write(|w| ..)` method takes [ienb::W](ienb::W) writer structure"]
impl crate::Writable for IENB {}
#[doc = "Port Interrupt B Enable"]
pub mod ienb;
#[doc = "Port Interrupt Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [int](int) module"]
pub type INT = crate::Reg<u16, _INT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INT;
#[doc = "`read()` method returns [int::R](int::R) reader structure"]
impl crate::Readable for INT {}
#[doc = "`write(|w| ..)` method takes [int::W](int::W) writer structure"]
impl crate::Writable for INT {}
#[doc = "Port Interrupt Status"]
pub mod int;
#[doc = "Port Drive Strength Select\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ds](ds) module"]
pub type DS = crate::Reg<u16, _DS>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DS;
#[doc = "`read()` method returns [ds::R](ds::R) reader structure"]
impl crate::Readable for DS {}
#[doc = "`write(|w| ..)` method takes [ds::W](ds::W) writer structure"]
impl crate::Writable for DS {}
#[doc = "Port Drive Strength Select"]
pub mod ds;
