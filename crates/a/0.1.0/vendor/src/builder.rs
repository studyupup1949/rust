use crate::frame;

/// Builder for the `CP=&&...&&` section.
///
/// This helper focuses on *string assembly* (HJ212 is ASCII). It intentionally does not
/// prescribe numeric formatting; callers pass values as strings.
#[derive(Debug, Default, Clone)]
pub struct CpBuilder {
    fields: Vec<String>,
}

impl CpBuilder {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Set `DataTime=YYYYMMDDhhmmss`.
    pub fn data_time(&mut self, data_time: impl AsRef<str>) -> &mut Self {
        self.kv("DataTime", data_time);
        self
    }

    /// Append a raw `key=value` field.
    pub fn kv(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> &mut Self {
        let k = key.as_ref().trim();
        let v = value.as_ref().trim();
        if k.is_empty() {
            return self;
        }
        self.fields.push(format!("{}={}", k, v));
        self
    }

    /// Append a pair of `*-Rtd` and `*-Flag` fields in the common format used by platforms.
    ///
    /// Example output fragment:
    /// `a21026-Rtd=12.3,a21026-Flag=N`
    ///
    /// Note: the default CP parser only splits fields on `;`. The comma here is intentionally
    /// left for the business layer to interpret if needed.
    pub fn rtd_flag(
        &mut self,
        code: impl AsRef<str>,
        rtd_value: impl AsRef<str>,
        flag: impl AsRef<str>,
    ) -> &mut Self {
        let c = code.as_ref().trim();
        if c.is_empty() {
            return self;
        }
        let rtd = rtd_value.as_ref().trim();
        let flg = flag.as_ref().trim();

        self.fields
            .push(format!("{}-Rtd={},{}-Flag={}", c, rtd, c, flg));
        self
    }

    /// Build the `CP` body string (without `CP=&&` wrappers).
    pub fn build(&self) -> String {
        if self.fields.is_empty() {
            return String::new();
        }
        // HJ212 commonly uses `;` between fields.
        // Ensure a trailing `;` for compatibility with many receivers.
        let mut out = self.fields.join(";");
        out.push(';');
        out
    }
}

/// Builder for the main HJ212 payload section (everything between `##{LEN}` and `{CRC}`),
/// plus convenience methods to wrap into a full frame.
#[derive(Debug, Clone)]
pub struct PayloadBuilder {
    qn: String,
    st: String,
    cn: String,
    pw: String,
    mn: String,
    flag: String,
    pnum: Option<String>,
    pno: Option<String>,
    cp_body: String,
}

impl PayloadBuilder {
    /// Create a payload builder with commonly used defaults for environmental monitoring.
    ///
    /// Defaults:
    /// - `ST=22`
    /// - `CN=2011`
    /// - `Flag=7`
    pub fn new(qn: impl AsRef<str>, pw: impl AsRef<str>, mn: impl AsRef<str>, cp_body: impl Into<String>) -> Self {
        Self {
            qn: qn.as_ref().trim().to_string(),
            st: "22".to_string(),
            cn: "2011".to_string(),
            pw: pw.as_ref().trim().to_string(),
            mn: mn.as_ref().trim().to_string(),
            flag: "7".to_string(),
            pnum: None,
            pno: None,
            cp_body: cp_body.into(),
        }
    }

    pub fn st(mut self, st: impl AsRef<str>) -> Self {
        self.st = st.as_ref().trim().to_string();
        self
    }

    pub fn cn(mut self, cn: impl AsRef<str>) -> Self {
        self.cn = cn.as_ref().trim().to_string();
        self
    }

    pub fn flag(mut self, flag: impl AsRef<str>) -> Self {
        self.flag = flag.as_ref().trim().to_string();
        self
    }

    /// Set `PNUM` (total packet count) for split packets.
    pub fn pnum(mut self, pnum: impl AsRef<str>) -> Self {
        self.pnum = Some(pnum.as_ref().trim().to_string());
        self
    }

    /// Set `PNO` (packet index, 1-based) for split packets.
    pub fn pno(mut self, pno: impl AsRef<str>) -> Self {
        self.pno = Some(pno.as_ref().trim().to_string());
        self
    }

    pub fn payload(&self) -> String {
        // Keep field order stable for easier debugging.
        // Always wrap CP as `CP=&&...&&` even if empty.
        let mut out = format!(
            "QN={};ST={};CN={};PW={};MN={};Flag={};",
            self.qn, self.st, self.cn, self.pw, self.mn, self.flag
        );
        if let Some(pnum) = &self.pnum {
            out.push_str(&format!("PNUM={};", pnum));
        }
        if let Some(pno) = &self.pno {
            out.push_str(&format!("PNO={};", pno));
        }
        out.push_str(&format!("CP=&&{}&&", self.cp_body));
        out
    }

    pub fn frame(&self) -> String {
        frame::build_frame(&self.payload())
    }

    /// Build a **standard** frame (CRC uppercase + trailing CRLF).
    pub fn frame_standard(&self) -> String {
        frame::build_frame_standard(&self.payload())
    }
}

/// Build a standard "请求应答" message (CN=9011) with `CP=&&QnRtn=...&&`.
///
/// Appendix C shows these responses commonly use `ST=91` and `Flag=8`.
pub fn build_qn_rtn(qn: impl AsRef<str>, pw: impl AsRef<str>, mn: impl AsRef<str>, qn_rtn: impl AsRef<str>) -> PayloadBuilder {
    let mut cp = CpBuilder::new();
    cp.kv("QnRtn", qn_rtn);
    PayloadBuilder::new(qn, pw, mn, cp.build())
        .st("91")
        .cn("9011")
        .flag("8")
}

/// Build a standard "执行结果" message (CN=9012) with `CP=&&ExeRtn=...&&`.
pub fn build_exe_rtn(qn: impl AsRef<str>, pw: impl AsRef<str>, mn: impl AsRef<str>, exe_rtn: impl AsRef<str>) -> PayloadBuilder {
    let mut cp = CpBuilder::new();
    cp.kv("ExeRtn", exe_rtn);
    PayloadBuilder::new(qn, pw, mn, cp.build())
        .st("91")
        .cn("9012")
        .flag("8")
}

/// Build a standard "数据应答" message (CN=9014) with an empty CP.
pub fn build_data_ack(qn: impl AsRef<str>, pw: impl AsRef<str>, mn: impl AsRef<str>) -> PayloadBuilder {
    PayloadBuilder::new(qn, pw, mn, String::new())
        .st("91")
        .cn("9014")
        .flag("8")
}

/// Build a standard "通知应答" message (CN=9013) with an empty CP.
pub fn build_notify_ack(qn: impl AsRef<str>, pw: impl AsRef<str>, mn: impl AsRef<str>) -> PayloadBuilder {
    PayloadBuilder::new(qn, pw, mn, String::new())
        .st("91")
        .cn("9013")
        .flag("8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cp_builder_renders_with_trailing_semicolon() {
        let mut cp = CpBuilder::new();
        cp.data_time("20250101010101")
            .rtd_flag("a21026", "12.3", "N")
            .kv("X", "1");
        let s = cp.build();
        assert!(s.starts_with("DataTime=20250101010101;"));
        assert!(s.contains("a21026-Rtd=12.3,a21026-Flag=N"));
        assert!(s.ends_with(';'));
    }

    #[test]
    fn payload_builder_wraps_cp_and_builds_frame() {
        let mut cp = CpBuilder::new();
        cp.data_time("20250101010101")
            .rtd_flag("a21026", "12.3", "N");

        let pb = PayloadBuilder::new("QN1", "123456", "ABC", cp.build());
        let payload = pb.payload();
        assert!(payload.contains("QN=QN1;"));
        assert!(payload.contains("ST=22;"));
        assert!(payload.contains("CN=2011;"));
        assert!(payload.contains("PW=123456;"));
        assert!(payload.contains("MN=ABC;"));
        assert!(payload.contains("Flag=7;"));
        assert!(payload.contains("CP=&&DataTime=20250101010101;"));

        let frame = pb.frame();
        // Basic sanity: starts with ## + 4 digits.
        assert!(frame.starts_with("##"));
        assert!(frame.len() > 10);
    }

    #[test]
    fn payload_builder_supports_pnum_pno_order() {
        let pb = PayloadBuilder::new("QN1", "123456", "MN1", "".to_string())
            .st("32")
            .cn("2061")
            .flag("11")
            .pnum("2")
            .pno("1");
        let payload = pb.payload();
        assert!(payload.contains("Flag=11;PNUM=2;PNO=1;CP=&&&&"));
    }

    #[test]
    fn appendix_c_ack_payloads_match_shape() {
        let qn_rtn = build_qn_rtn("20240601085857223", "123456", "010000A8900016F000169DC0", "1").payload();
        assert!(qn_rtn.contains("ST=91;CN=9011;"));
        assert!(qn_rtn.contains("Flag=8;"));
        assert!(qn_rtn.contains("CP=&&QnRtn=1;&&"));

        let exe_rtn = build_exe_rtn("20240601085857223", "123456", "010000A8900016F000169DC0", "1").payload();
        assert!(exe_rtn.contains("ST=91;CN=9012;"));
        assert!(exe_rtn.contains("CP=&&ExeRtn=1;&&"));

        let ack = build_data_ack("20240601085857534", "123456", "010000A8900016F000169DC0").payload();
        assert!(ack.contains("ST=91;CN=9014;"));
        assert!(ack.ends_with("CP=&&&&"));
    }
}
