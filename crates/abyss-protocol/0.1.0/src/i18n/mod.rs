use std::collections::HashMap;
use anyhow::Result;

const EN_TOML: &str = include_str!("../../locales/en.toml");
const ZH_TOML: &str = include_str!("../../locales/zh.toml");

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Locale {
    En,
    Zh,
}

pub struct LocaleManager {
    current_locale: Locale,
    strings: HashMap<String, String>,
}

impl LocaleManager {
    /// 从嵌入的 TOML 内容加载指定语言
    pub fn load(locale: Locale) -> Result<Self> {
        let raw = match locale {
            Locale::En => EN_TOML,
            Locale::Zh => ZH_TOML,
        };
        let strings = Self::parse_toml(raw)?;
        Ok(Self {
            current_locale: locale,
            strings,
        })
    }

    /// 获取本地化字符串，key 不存在时返回 key 本身
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).map(|s| s.as_str()).unwrap_or(key)
    }

    /// 带参数的本地化字符串（{0}, {1} 占位符替换）
    pub fn tf(&self, key: &str, args: &[&str]) -> String {
        let template = self.t(key);
        let mut result = template.to_string();
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), arg);
        }
        result
    }

    /// 运行时切换语言
    pub fn switch_locale(&mut self, locale: Locale) -> Result<()> {
        let raw = match locale {
            Locale::En => EN_TOML,
            Locale::Zh => ZH_TOML,
        };
        self.strings = Self::parse_toml(raw)?;
        self.current_locale = locale;
        Ok(())
    }

    /// 获取当前语言
    pub fn current(&self) -> Locale {
        self.current_locale
    }

    /// 解析 TOML 内容为扁平化的 dot-separated key map
    fn parse_toml(raw: &str) -> Result<HashMap<String, String>> {
        let table: toml::Table = toml::from_str(raw)?;
        let mut map = HashMap::new();
        Self::flatten(&table, "", &mut map);
        Ok(map)
    }

    /// 递归扁平化 TOML 嵌套表为 "section.key" 格式
    fn flatten(table: &toml::Table, prefix: &str, out: &mut HashMap<String, String>) {
        for (key, value) in table {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };
            match value {
                toml::Value::String(s) => {
                    out.insert(full_key, s.clone());
                }
                toml::Value::Table(sub) => {
                    Self::flatten(sub, &full_key, out);
                }
                _ => {
                    // 非字符串、非表的值转为字符串存储
                    out.insert(full_key, value.to_string());
                }
            }
        }
    }
}

/// 从 CLI 参数、环境变量、系统 locale 自动检测语言
pub fn detect_locale(cli_lang: Option<&str>) -> Locale {
    // 优先级 1: --lang 参数
    if let Some(lang) = cli_lang {
        return match lang {
            "zh" => Locale::Zh,
            _ => Locale::En,
        };
    }

    // 优先级 2: ABYSS_LANG 环境变量
    if let Ok(lang) = std::env::var("ABYSS_LANG") {
        return match lang.as_str() {
            "zh" => Locale::Zh,
            _ => Locale::En,
        };
    }

    // 优先级 3: 系统 LANG 环境变量
    if let Ok(lang) = std::env::var("LANG") {
        if lang.contains("zh") {
            return Locale::Zh;
        }
    }

    Locale::En
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_en() {
        let lm = LocaleManager::load(Locale::En).unwrap();
        assert_eq!(lm.t("ui.title"), "PROTOCOL: C.L.A.U.D.E");
        assert_eq!(lm.t("tabs.altar"), "Altar");
        assert_eq!(lm.t("san_states.lucid"), "Lucid");
    }

    #[test]
    fn test_load_zh() {
        let lm = LocaleManager::load(Locale::Zh).unwrap();
        assert_eq!(lm.t("ui.title"), "协议：C.L.A.U.D.E");
        assert_eq!(lm.t("tabs.altar"), "祭坛");
        assert_eq!(lm.t("san_states.lucid"), "清醒");
    }

    #[test]
    fn test_missing_key_returns_key() {
        let lm = LocaleManager::load(Locale::En).unwrap();
        assert_eq!(lm.t("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_tf_with_args() {
        let lm = LocaleManager::load(Locale::En).unwrap();
        let result = lm.tf("ui.coming_spec2", &["Altar"]);
        assert_eq!(result, "Altar - Coming in Spec 2");
    }

    #[test]
    fn test_switch_locale() {
        let mut lm = LocaleManager::load(Locale::En).unwrap();
        assert_eq!(lm.current(), Locale::En);
        assert_eq!(lm.t("tabs.altar"), "Altar");

        lm.switch_locale(Locale::Zh).unwrap();
        assert_eq!(lm.current(), Locale::Zh);
        assert_eq!(lm.t("tabs.altar"), "祭坛");
    }

    #[test]
    fn test_detect_locale_cli_arg() {
        assert_eq!(detect_locale(Some("zh")), Locale::Zh);
        assert_eq!(detect_locale(Some("en")), Locale::En);
        assert_eq!(detect_locale(Some("fr")), Locale::En);
    }
}
