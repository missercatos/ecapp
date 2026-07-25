use crossterm::{
    cursor::{MoveTo, MoveToColumn, RestorePosition, SavePosition},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType},
};
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use ureq::Agent;

// ── help ──────────────────────────────────────────────────────────────

const HELP_TEXT: &str = r#"
ecapp — Terminal Translation Tool
==================================

Main Mode Commands:
  help, h                 Show this help
  api                     Manage translation API backend and keys
  translate, tra          Enter translation mode (guided setup)
  tra /<src> ~ <tgt>      Enter translation mode directly
                          Example: tra /en_us ~ zh_cn
  tra-dir, td             Enter dictionary-enhanced translation mode
                          Same as tra, but single-word lookups show
                          dictionary entries with phonetics.
  exit                    Exit ecapp

Translate Mode Commands:
  :/tip                   Show available languages
  :/reelect, ree          Re-select source and target languages
  :/source <code>         Change only the source language
  :/target <code>         Change only the target language
  :/swap                  Swap source and target languages
  :/help, :/h             Show this help
  :/exit                  Return to ecapp main mode

  Any other text is translated from source to target language.
  In dictionary mode (tra-dir), single words also show dictionary
  definitions with phonetics and multiple meanings.

Keyboard Shortcuts:
  Ctrl+U                  Clear the current input line
  Ctrl+C                  Interrupt / cancel

Language codes use Arch-style format: lowercase with underscore
(e.g., en_us, zh_cn, ja_jp, fr_fr, de_de, es_es, pt_br, ru_ru).
"#;

// ── language data ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Language {
    code: &'static str,
    api_code: &'static str,
    dict_code: &'static str,
    arch_locale: &'static str,
    english_name: &'static str,
    native_name: &'static str,
}

const LANGUAGES: &[Language] = &[
    Language { code: "en_us", api_code: "en", dict_code: "en", arch_locale: "en_US.UTF-8", english_name: "English (US)", native_name: "English (US)" },
    Language { code: "en_gb", api_code: "en", dict_code: "en", arch_locale: "en_GB.UTF-8", english_name: "English (UK)", native_name: "English (UK)" },
    Language { code: "zh_cn", api_code: "zh-CN", dict_code: "zh", arch_locale: "zh_CN.UTF-8", english_name: "Chinese (Simplified)", native_name: "简体中文" },
    Language { code: "zh_tw", api_code: "zh-TW", dict_code: "zh", arch_locale: "zh_TW.UTF-8", english_name: "Chinese (Traditional)", native_name: "繁體中文" },
    Language { code: "ja_jp", api_code: "ja", dict_code: "ja", arch_locale: "ja_JP.UTF-8", english_name: "Japanese", native_name: "日本語" },
    Language { code: "ko_kr", api_code: "ko", dict_code: "ko", arch_locale: "ko_KR.UTF-8", english_name: "Korean", native_name: "한국어" },
    Language { code: "fr_fr", api_code: "fr", dict_code: "fr", arch_locale: "fr_FR.UTF-8", english_name: "French", native_name: "Français" },
    Language { code: "de_de", api_code: "de", dict_code: "de", arch_locale: "de_DE.UTF-8", english_name: "German", native_name: "Deutsch" },
    Language { code: "es_es", api_code: "es", dict_code: "es", arch_locale: "es_ES.UTF-8", english_name: "Spanish", native_name: "Español" },
    Language { code: "pt_br", api_code: "pt", dict_code: "pt", arch_locale: "pt_BR.UTF-8", english_name: "Portuguese (Brazil)", native_name: "Português (Brasil)" },
    Language { code: "pt_pt", api_code: "pt", dict_code: "pt", arch_locale: "pt_PT.UTF-8", english_name: "Portuguese (Portugal)", native_name: "Português (Portugal)" },
    Language { code: "ru_ru", api_code: "ru", dict_code: "ru", arch_locale: "ru_RU.UTF-8", english_name: "Russian", native_name: "Русский" },
    Language { code: "it_it", api_code: "it", dict_code: "it", arch_locale: "it_IT.UTF-8", english_name: "Italian", native_name: "Italiano" },
    Language { code: "ar_sa", api_code: "ar", dict_code: "ar", arch_locale: "ar_SA.UTF-8", english_name: "Arabic", native_name: "العربية" },
    Language { code: "hi_in", api_code: "hi", dict_code: "hi", arch_locale: "hi_IN.UTF-8", english_name: "Hindi", native_name: "हिन्दी" },
    Language { code: "th_th", api_code: "th", dict_code: "th", arch_locale: "th_TH.UTF-8", english_name: "Thai", native_name: "ไทย" },
    Language { code: "vi_vn", api_code: "vi", dict_code: "vi", arch_locale: "vi_VN.UTF-8", english_name: "Vietnamese", native_name: "Tiếng Việt" },
    Language { code: "nl_nl", api_code: "nl", dict_code: "nl", arch_locale: "nl_NL.UTF-8", english_name: "Dutch", native_name: "Nederlands" },
    Language { code: "pl_pl", api_code: "pl", dict_code: "pl", arch_locale: "pl_PL.UTF-8", english_name: "Polish", native_name: "Polski" },
    Language { code: "sv_se", api_code: "sv", dict_code: "sv", arch_locale: "sv_SE.UTF-8", english_name: "Swedish", native_name: "Svenska" },
    Language { code: "tr_tr", api_code: "tr", dict_code: "tr", arch_locale: "tr_TR.UTF-8", english_name: "Turkish", native_name: "Türkçe" },
    Language { code: "uk_ua", api_code: "uk", dict_code: "uk", arch_locale: "uk_UA.UTF-8", english_name: "Ukrainian", native_name: "Українська" },
    Language { code: "cs_cz", api_code: "cs", dict_code: "cs", arch_locale: "cs_CZ.UTF-8", english_name: "Czech", native_name: "Čeština" },
    Language { code: "ro_ro", api_code: "ro", dict_code: "ro", arch_locale: "ro_RO.UTF-8", english_name: "Romanian", native_name: "Română" },
    Language { code: "el_gr", api_code: "el", dict_code: "el", arch_locale: "el_GR.UTF-8", english_name: "Greek", native_name: "Ελληνικά" },
    Language { code: "he_il", api_code: "he", dict_code: "he", arch_locale: "he_IL.UTF-8", english_name: "Hebrew", native_name: "עברית" },
    Language { code: "hu_hu", api_code: "hu", dict_code: "hu", arch_locale: "hu_HU.UTF-8", english_name: "Hungarian", native_name: "Magyar" },
    Language { code: "fi_fi", api_code: "fi", dict_code: "fi", arch_locale: "fi_FI.UTF-8", english_name: "Finnish", native_name: "Suomi" },
    Language { code: "da_dk", api_code: "da", dict_code: "da", arch_locale: "da_DK.UTF-8", english_name: "Danish", native_name: "Dansk" },
    Language { code: "nb_no", api_code: "nb", dict_code: "no", arch_locale: "nb_NO.UTF-8", english_name: "Norwegian (Bokmål)", native_name: "Norsk (Bokmål)" },
    Language { code: "sk_sk", api_code: "sk", dict_code: "sk", arch_locale: "sk_SK.UTF-8", english_name: "Slovak", native_name: "Slovenčina" },
    Language { code: "bg_bg", api_code: "bg", dict_code: "bg", arch_locale: "bg_BG.UTF-8", english_name: "Bulgarian", native_name: "Български" },
    Language { code: "hr_hr", api_code: "hr", dict_code: "hr", arch_locale: "hr_HR.UTF-8", english_name: "Croatian", native_name: "Hrvatski" },
    Language { code: "lt_lt", api_code: "lt", dict_code: "lt", arch_locale: "lt_LT.UTF-8", english_name: "Lithuanian", native_name: "Lietuvių" },
    Language { code: "lv_lv", api_code: "lv", dict_code: "lv", arch_locale: "lv_LV.UTF-8", english_name: "Latvian", native_name: "Latviešu" },
    Language { code: "et_ee", api_code: "et", dict_code: "et", arch_locale: "et_EE.UTF-8", english_name: "Estonian", native_name: "Eesti" },
    Language { code: "sl_si", api_code: "sl", dict_code: "sl", arch_locale: "sl_SI.UTF-8", english_name: "Slovenian", native_name: "Slovenščina" },
    Language { code: "ms_my", api_code: "ms", dict_code: "ms", arch_locale: "ms_MY.UTF-8", english_name: "Malay", native_name: "Bahasa Melayu" },
    Language { code: "id_id", api_code: "id", dict_code: "id", arch_locale: "id_ID.UTF-8", english_name: "Indonesian", native_name: "Bahasa Indonesia" },
    Language { code: "tl_ph", api_code: "tl", dict_code: "tl", arch_locale: "tl_PH.UTF-8", english_name: "Filipino (Tagalog)", native_name: "Filipino" },
    Language { code: "sw_ke", api_code: "sw", dict_code: "sw", arch_locale: "sw_KE.UTF-8", english_name: "Swahili", native_name: "Kiswahili" },
    Language { code: "af_za", api_code: "af", dict_code: "af", arch_locale: "af_ZA.UTF-8", english_name: "Afrikaans", native_name: "Afrikaans" },
    Language { code: "sq_al", api_code: "sq", dict_code: "sq", arch_locale: "sq_AL.UTF-8", english_name: "Albanian", native_name: "Shqip" },
    Language { code: "am_et", api_code: "am", dict_code: "am", arch_locale: "am_ET.UTF-8", english_name: "Amharic", native_name: "አማርኛ" },
    Language { code: "hy_am", api_code: "hy", dict_code: "hy", arch_locale: "hy_AM.UTF-8", english_name: "Armenian", native_name: "Հայերեն" },
    Language { code: "az_az", api_code: "az", dict_code: "az", arch_locale: "az_AZ.UTF-8", english_name: "Azerbaijani", native_name: "Azərbaycan" },
    Language { code: "eu_es", api_code: "eu", dict_code: "eu", arch_locale: "eu_ES.UTF-8", english_name: "Basque", native_name: "Euskara" },
    Language { code: "be_by", api_code: "be", dict_code: "be", arch_locale: "be_BY.UTF-8", english_name: "Belarusian", native_name: "Беларуская" },
    Language { code: "bn_in", api_code: "bn", dict_code: "bn", arch_locale: "bn_IN.UTF-8", english_name: "Bengali", native_name: "বাংলা" },
    Language { code: "bs_ba", api_code: "bs", dict_code: "bs", arch_locale: "bs_BA.UTF-8", english_name: "Bosnian", native_name: "Bosanski" },
    Language { code: "ca_es", api_code: "ca", dict_code: "ca", arch_locale: "ca_ES.UTF-8", english_name: "Catalan", native_name: "Català" },
    Language { code: "ceb_ph", api_code: "ceb", dict_code: "ceb", arch_locale: "ceb_PH.UTF-8", english_name: "Cebuano", native_name: "Cebuano" },
    Language { code: "co_fr", api_code: "co", dict_code: "co", arch_locale: "co_FR.UTF-8", english_name: "Corsican", native_name: "Corsu" },
    Language { code: "cy_gb", api_code: "cy", dict_code: "cy", arch_locale: "cy_GB.UTF-8", english_name: "Welsh", native_name: "Cymraeg" },
    Language { code: "eo_xx", api_code: "eo", dict_code: "eo", arch_locale: "eo.UTF-8", english_name: "Esperanto", native_name: "Esperanto" },
    Language { code: "ga_ie", api_code: "ga", dict_code: "ga", arch_locale: "ga_IE.UTF-8", english_name: "Irish", native_name: "Gaeilge" },
    Language { code: "gl_es", api_code: "gl", dict_code: "gl", arch_locale: "gl_ES.UTF-8", english_name: "Galician", native_name: "Galego" },
    Language { code: "gu_in", api_code: "gu", dict_code: "gu", arch_locale: "gu_IN.UTF-8", english_name: "Gujarati", native_name: "ગુજરાતી" },
    Language { code: "ha_ng", api_code: "ha", dict_code: "ha", arch_locale: "ha_NG.UTF-8", english_name: "Hausa", native_name: "Hausa" },
    Language { code: "haw_us", api_code: "haw", dict_code: "haw", arch_locale: "haw_US.UTF-8", english_name: "Hawaiian", native_name: "ʻŌlelo Hawaiʻi" },
    Language { code: "hmn_cn", api_code: "hmn", dict_code: "hmn", arch_locale: "hmn_CN.UTF-8", english_name: "Hmong", native_name: "Hmoob" },
    Language { code: "ht_ht", api_code: "ht", dict_code: "ht", arch_locale: "ht_HT.UTF-8", english_name: "Haitian Creole", native_name: "Kreyòl Ayisyen" },
    Language { code: "ig_ng", api_code: "ig", dict_code: "ig", arch_locale: "ig_NG.UTF-8", english_name: "Igbo", native_name: "Igbo" },
    Language { code: "is_is", api_code: "is", dict_code: "is", arch_locale: "is_IS.UTF-8", english_name: "Icelandic", native_name: "Íslenska" },
    Language { code: "jv_id", api_code: "jv", dict_code: "jv", arch_locale: "jv_ID.UTF-8", english_name: "Javanese", native_name: "Basa Jawa" },
    Language { code: "ka_ge", api_code: "ka", dict_code: "ka", arch_locale: "ka_GE.UTF-8", english_name: "Georgian", native_name: "ქართული" },
    Language { code: "kk_kz", api_code: "kk", dict_code: "kk", arch_locale: "kk_KZ.UTF-8", english_name: "Kazakh", native_name: "Қазақ" },
    Language { code: "km_kh", api_code: "km", dict_code: "km", arch_locale: "km_KH.UTF-8", english_name: "Khmer", native_name: "ខ្មែរ" },
    Language { code: "kn_in", api_code: "kn", dict_code: "kn", arch_locale: "kn_IN.UTF-8", english_name: "Kannada", native_name: "ಕನ್ನಡ" },
    Language { code: "ku_tr", api_code: "ku", dict_code: "ku", arch_locale: "ku_TR.UTF-8", english_name: "Kurdish", native_name: "Kurdî" },
    Language { code: "ky_kg", api_code: "ky", dict_code: "ky", arch_locale: "ky_KG.UTF-8", english_name: "Kyrgyz", native_name: "Кыргызча" },
    Language { code: "la_va", api_code: "la", dict_code: "la", arch_locale: "la_VA.UTF-8", english_name: "Latin", native_name: "Latina" },
    Language { code: "lb_lu", api_code: "lb", dict_code: "lb", arch_locale: "lb_LU.UTF-8", english_name: "Luxembourgish", native_name: "Lëtzebuergesch" },
    Language { code: "lo_la", api_code: "lo", dict_code: "lo", arch_locale: "lo_LA.UTF-8", english_name: "Lao", native_name: "ລາວ" },
    Language { code: "mk_mk", api_code: "mk", dict_code: "mk", arch_locale: "mk_MK.UTF-8", english_name: "Macedonian", native_name: "Македонски" },
    Language { code: "mg_mg", api_code: "mg", dict_code: "mg", arch_locale: "mg_MG.UTF-8", english_name: "Malagasy", native_name: "Malagasy" },
    Language { code: "ml_in", api_code: "ml", dict_code: "ml", arch_locale: "ml_IN.UTF-8", english_name: "Malayalam", native_name: "മലയാളം" },
    Language { code: "mt_mt", api_code: "mt", dict_code: "mt", arch_locale: "mt_MT.UTF-8", english_name: "Maltese", native_name: "Malti" },
    Language { code: "mi_nz", api_code: "mi", dict_code: "mi", arch_locale: "mi_NZ.UTF-8", english_name: "Māori", native_name: "Te Reo Māori" },
    Language { code: "mr_in", api_code: "mr", dict_code: "mr", arch_locale: "mr_IN.UTF-8", english_name: "Marathi", native_name: "मराठी" },
    Language { code: "mn_mn", api_code: "mn", dict_code: "mn", arch_locale: "mn_MN.UTF-8", english_name: "Mongolian", native_name: "Монгол" },
    Language { code: "my_mm", api_code: "my", dict_code: "my", arch_locale: "my_MM.UTF-8", english_name: "Burmese", native_name: "မြန်မာ" },
    Language { code: "ne_np", api_code: "ne", dict_code: "ne", arch_locale: "ne_NP.UTF-8", english_name: "Nepali", native_name: "नेपाली" },
    Language { code: "ny_mw", api_code: "ny", dict_code: "ny", arch_locale: "ny_MW.UTF-8", english_name: "Chichewa (Nyanja)", native_name: "Chichewa" },
    Language { code: "or_in", api_code: "or", dict_code: "or", arch_locale: "or_IN.UTF-8", english_name: "Odia", native_name: "ଓଡ଼ିଆ" },
    Language { code: "pa_in", api_code: "pa", dict_code: "pa", arch_locale: "pa_IN.UTF-8", english_name: "Punjabi", native_name: "ਪੰਜਾਬੀ" },
    Language { code: "fa_ir", api_code: "fa", dict_code: "fa", arch_locale: "fa_IR.UTF-8", english_name: "Persian", native_name: "فارسی" },
    Language { code: "ps_af", api_code: "ps", dict_code: "ps", arch_locale: "ps_AF.UTF-8", english_name: "Pashto", native_name: "پښتو" },
    Language { code: "sm_ws", api_code: "sm", dict_code: "sm", arch_locale: "sm_WS.UTF-8", english_name: "Samoan", native_name: "Gagana Samoa" },
    Language { code: "gd_gb", api_code: "gd", dict_code: "gd", arch_locale: "gd_GB.UTF-8", english_name: "Scots Gaelic", native_name: "Gàidhlig" },
    Language { code: "sr_rs", api_code: "sr", dict_code: "sr", arch_locale: "sr_RS.UTF-8", english_name: "Serbian", native_name: "Српски" },
    Language { code: "sn_zw", api_code: "sn", dict_code: "sn", arch_locale: "sn_ZW.UTF-8", english_name: "Shona", native_name: "chiShona" },
    Language { code: "sd_pk", api_code: "sd", dict_code: "sd", arch_locale: "sd_PK.UTF-8", english_name: "Sindhi", native_name: "سنڌي" },
    Language { code: "si_lk", api_code: "si", dict_code: "si", arch_locale: "si_LK.UTF-8", english_name: "Sinhala", native_name: "සිංහල" },
    Language { code: "so_so", api_code: "so", dict_code: "so", arch_locale: "so_SO.UTF-8", english_name: "Somali", native_name: "Soomaali" },
    Language { code: "st_za", api_code: "st", dict_code: "st", arch_locale: "st_ZA.UTF-8", english_name: "Sesotho", native_name: "Sesotho" },
    Language { code: "su_id", api_code: "su", dict_code: "su", arch_locale: "su_ID.UTF-8", english_name: "Sundanese", native_name: "Basa Sunda" },
    Language { code: "tg_tj", api_code: "tg", dict_code: "tg", arch_locale: "tg_TJ.UTF-8", english_name: "Tajik", native_name: "Тоҷикӣ" },
    Language { code: "ta_in", api_code: "ta", dict_code: "ta", arch_locale: "ta_IN.UTF-8", english_name: "Tamil", native_name: "தமிழ்" },
    Language { code: "te_in", api_code: "te", dict_code: "te", arch_locale: "te_IN.UTF-8", english_name: "Telugu", native_name: "తెలుగు" },
    Language { code: "ur_pk", api_code: "ur", dict_code: "ur", arch_locale: "ur_PK.UTF-8", english_name: "Urdu", native_name: "اردو" },
    Language { code: "uz_uz", api_code: "uz", dict_code: "uz", arch_locale: "uz_UZ.UTF-8", english_name: "Uzbek", native_name: "Oʻzbek" },
    Language { code: "xh_za", api_code: "xh", dict_code: "xh", arch_locale: "xh_ZA.UTF-8", english_name: "Xhosa", native_name: "isiXhosa" },
    Language { code: "yi_us", api_code: "yi", dict_code: "yi", arch_locale: "yi_US.UTF-8", english_name: "Yiddish", native_name: "ייִדיש" },
    Language { code: "yo_ng", api_code: "yo", dict_code: "yo", arch_locale: "yo_NG.UTF-8", english_name: "Yoruba", native_name: "Yorùbá" },
    Language { code: "zu_za", api_code: "zu", dict_code: "zu", arch_locale: "zu_ZA.UTF-8", english_name: "Zulu", native_name: "isiZulu" },
];

fn find_language(code: &str) -> Option<&Language> {
    LANGUAGES.iter().find(|l| l.code.eq_ignore_ascii_case(code))
}

// ── api config ────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ApiConfig {
    backend: String,
    api_key: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self { backend: "mymemory".into(), api_key: None }
    }
}

const BACKENDS: &[(&str, &str, &str)] = &[
    ("mymemory", "MyMemory",        "Free, no key required"),
    ("google",   "Google Translate", "Needs API key (Cloud Translation)"),
    ("deepl",    "DeepL",            "Needs API key (free tier: 500k chars/mo)"),
];

fn config_path() -> PathBuf {
    let mut p = dirs_home();
    p.push(".config");
    p.push("ecapp");
    p
}

fn config_file() -> PathBuf {
    config_path().join("config.json")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn load_config() -> ApiConfig {
    let path = config_file();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(cfg: &ApiConfig) {
    let dir = config_path();
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(config_file(), serde_json::to_string_pretty(cfg).unwrap_or_default());
}

// ── url helpers ───────────────────────────────────────────────────────

fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                use std::fmt::Write;
                let _ = write!(result, "%{:02X}", byte);
            }
        }
    }
    result
}

// ── mymemory translation API ──────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct MmResponse {
    #[serde(rename = "responseData")]
    response_data: MmData,
    #[serde(rename = "responseStatus")]
    response_status: Option<i32>,
}

#[derive(Deserialize, Debug)]
struct MmData {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

fn translate_via_mymemory(
    agent: &Agent,
    text: &str,
    source_code: &str,
    target_code: &str,
) -> Result<String, String> {
    let src = find_language(source_code)
        .ok_or_else(|| format!("unknown source language '{source_code}'"))?;
    let tgt = find_language(target_code)
        .ok_or_else(|| format!("unknown target language '{target_code}'"))?;

    let url = format!(
        "https://api.mymemory.translated.net/get?q={}&langpair={}|{}",
        url_encode(text),
        src.api_code,
        tgt.api_code,
    );

    let resp = agent
        .get(&url)
        .header("User-Agent", "ecapp/0.2")
        .call()
        .map_err(|e| format!("network error: {e}"))?;

    let mut body_bytes = Vec::new();
    resp.into_body()
        .as_reader()
        .read_to_end(&mut body_bytes)
        .map_err(|e| format!("read error: {e}"))?;

    let body: MmResponse =
        serde_json::from_slice(&body_bytes).map_err(|e| format!("parse error: {e}"))?;

    match body.response_status {
        Some(200) | Some(202) | None => Ok(body.response_data.translated_text),
        Some(s) => Err(format!("API error (status {s})")),
    }
}

fn translate_google(
    agent: &Agent,
    api_key: &str,
    text: &str,
    source_code: &str,
    target_code: &str,
) -> Result<String, String> {
    let src = find_language(source_code)
        .ok_or_else(|| format!("unknown source language '{source_code}'"))?;
    let tgt = find_language(target_code)
        .ok_or_else(|| format!("unknown target language '{target_code}'"))?;

    let body = serde_json::json!({
        "q": text,
        "source": src.api_code,
        "target": tgt.api_code,
        "format": "text",
    });

    let url = format!(
        "https://translation.googleapis.com/language/translate/v2?key={api_key}"
    );

    let resp = agent
        .post(&url)
        .header("Content-Type", "application/json; charset=utf-8")
        .send_json(&body)
        .map_err(|e| format!("network error: {e}"))?;

    #[derive(Deserialize)]
    struct GTransResp {
        data: GTransData,
    }
    #[derive(Deserialize)]
    struct GTransData {
        translations: Vec<GTransEntry>,
    }
    #[derive(Deserialize)]
    struct GTransEntry {
        #[serde(rename = "translatedText")]
        translated_text: String,
    }

    let mut buf = Vec::new();
    resp.into_body().as_reader().read_to_end(&mut buf)
        .map_err(|e| format!("read error: {e}"))?;
    let g: GTransResp = serde_json::from_slice(&buf)
        .map_err(|e| format!("parse error: {e}"))?;
    g.data.translations
        .into_iter()
        .next()
        .map(|t| t.translated_text)
        .ok_or_else(|| "no translation".into())
}

fn translate_deepl(
    agent: &Agent,
    api_key: &str,
    text: &str,
    source_code: &str,
    target_code: &str,
) -> Result<String, String> {
    let src = find_language(source_code)
        .ok_or_else(|| format!("unknown source language '{source_code}'"))?;
    let tgt = find_language(target_code)
        .ok_or_else(|| format!("unknown target language '{target_code}'"))?;

    let src_upper = src.api_code.to_uppercase();
    let tgt_upper = tgt.api_code.to_uppercase();

    let url = "https://api-free.deepl.com/v2/translate";
    let params: Vec<(&str, &str)> = vec![
        ("text", text),
        ("source_lang", &src_upper),
        ("target_lang", &tgt_upper),
    ];

    let resp = agent
        .post(url)
        .header("Authorization", &format!("DeepL-Auth-Key {api_key}"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send_form(params)
        .map_err(|e| format!("network error: {e}"))?;

    #[derive(Deserialize)]
    struct DeepLResp {
        translations: Vec<DeepLEntry>,
    }
    #[derive(Deserialize)]
    struct DeepLEntry {
        text: String,
    }

    let mut buf = Vec::new();
    resp.into_body().as_reader().read_to_end(&mut buf)
        .map_err(|e| format!("read error: {e}"))?;
    let d: DeepLResp = serde_json::from_slice(&buf)
        .map_err(|e| format!("parse error: {e}"))?;
    d.translations
        .into_iter()
        .next()
        .map(|t| t.text)
        .ok_or_else(|| "no translation".into())
}

fn translate_dispatch(
    agent: &Agent,
    config: &ApiConfig,
    text: &str,
    source: &str,
    target: &str,
) -> Result<String, String> {
    match config.backend.as_str() {
        "google" => {
            let key = config.api_key.as_deref().unwrap_or("");
            if key.is_empty() {
                return Err("Google API key not set. Use 'api' command to configure.".into());
            }
            translate_google(agent, key, text, source, target)
        }
        "deepl" => {
            let key = config.api_key.as_deref().unwrap_or("");
            if key.is_empty() {
                return Err("DeepL API key not set. Use 'api' command to configure.".into());
            }
            translate_deepl(agent, key, text, source, target)
        }
        _ => translate_via_mymemory(agent, text, source, target),
    }
}

// ── free dictionary API ───────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct DictEntry {
    word: String,
    phonetic: Option<String>,
    phonetics: Option<Vec<DictPhonetic>>,
    meanings: Option<Vec<DictMeaning>>,
}

#[derive(Deserialize, Debug)]
struct DictPhonetic {
    text: Option<String>,
    #[allow(dead_code)]
    audio: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "sourceUrl")]
    _source_url: Option<String>,
    #[allow(dead_code)]
    license: Option<DictLicense>,
}

#[derive(Deserialize, Debug)]
struct DictLicense {
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DictMeaning {
    #[serde(rename = "partOfSpeech")]
    part_of_speech: Option<String>,
    definitions: Option<Vec<DictDefinition>>,
}

#[derive(Deserialize, Debug)]
struct DictDefinition {
    definition: Option<String>,
    example: Option<String>,
    synonyms: Option<Vec<String>>,
    #[allow(dead_code)]
    antonyms: Option<Vec<String>>,
}

fn lookup_dictionary(
    agent: &Agent,
    word: &str,
    source_code: &str,
) -> Result<Vec<DictEntry>, String> {
    let lang = find_language(source_code)
        .ok_or_else(|| format!("unknown language '{source_code}'"))?;

    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/{}/{}",
        lang.dict_code,
        url_encode(word),
    );

    let resp = agent
        .get(&url)
        .header("User-Agent", "ecapp/0.2")
        .call()
        .map_err(|e| {
            if e.to_string().contains("404") || e.to_string().contains("status 404") {
                "no dictionary entry found".to_string()
            } else {
                format!("network error: {e}")
            }
        })?;

    let mut body_bytes = Vec::new();
    resp.into_body()
        .as_reader()
        .read_to_end(&mut body_bytes)
        .map_err(|e| format!("read error: {e}"))?;

    let entries: Vec<DictEntry> =
        serde_json::from_slice(&body_bytes).map_err(|e| format!("parse error: {e}"))?;

    if entries.is_empty() {
        Err("no dictionary entry found".into())
    } else {
        Ok(entries)
    }
}

fn format_dictionary(entries: &[DictEntry]) -> String {
    let mut out = String::new();

    for entry in entries {
        use std::fmt::Write;
        let _ = writeln!(out, "\x1b[1;33m{}\x1b[0m", entry.word);

        let phonetics: Vec<&str> = entry
            .phonetics
            .as_ref()
            .map(|p| {
                p.iter()
                    .filter_map(|ph| ph.text.as_deref())
                    .collect()
            })
            .unwrap_or_default();

        let phonetic_str: Vec<&str> = if !phonetics.is_empty() {
            phonetics
        } else if let Some(ref p) = entry.phonetic {
            vec![p.as_str()]
        } else {
            vec![]
        };

        if !phonetic_str.is_empty() {
            let _ = writeln!(out, "  \x1b[36m/{}/\x1b[0m", phonetic_str.join("  /"));
        }

        if let Some(ref meanings) = entry.meanings {
            for meaning in meanings {
                let pos = meaning
                    .part_of_speech
                    .as_deref()
                    .unwrap_or("—");
                let _ = writeln!(out, "  \x1b[1;32m{pos}\x1b[0m");

                if let Some(ref defs) = meaning.definitions {
                    for (i, def) in defs.iter().enumerate() {
                        let def_text = def.definition.as_deref().unwrap_or("—");
                        let _ = writeln!(out, "    {}. {}", i + 1, def_text);

                        if let Some(ref ex) = def.example {
                            let _ = writeln!(out, "       \x1b[90m\"{ex}\"\x1b[0m");
                        }
                        if let Some(ref syns) = def.synonyms
                            && !syns.is_empty() {
                                let _ = writeln!(out, "       \x1b[33msynonyms:\x1b[0m {}", syns.join(", "));
                            }
                    }
                }
            }
        }
        let _ = writeln!(out);
    }
    out
}

fn is_single_word(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty() && !trimmed.contains(' ')
}

// ── tip display ───────────────────────────────────────────────────────

fn show_tip() {
    println!();
    println!(
        "  {:<12} {:<34} {:<30} {:<18}",
        "CODE", "ENGLISH NAME", "NATIVE NAME", "ARCH LOCALE"
    );
    println!(
        "  {:-<12} {:-<34} {:-<30} {:-<18}",
        "", "", "", ""
    );
    for lang in LANGUAGES {
        println!(
            "  {:<12} {:<34} {:<30} {:<18}",
            lang.code, lang.english_name, lang.native_name, lang.arch_locale
        );
    }
    println!();
    println!("  Use the CODE value when selecting languages.");
    println!();
}

// ── prompt helpers ────────────────────────────────────────────────────

fn print_prompt(src: &str, tgt: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        MoveToColumn(0),
        Clear(ClearType::CurrentLine),
        Print(format!("\x1b[1;34m[Translate:\x1b[0m {src} \x1b[1;34m->\x1b[0m {tgt}\x1b[1;34m]\x1b[0m ")),
    )?;
    stdout.flush()?;
    Ok(())
}

fn print_prompt_dict(src: &str, tgt: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        MoveToColumn(0),
        Clear(ClearType::CurrentLine),
        Print(format!("\x1b[1;35m[Dict:\x1b[0m {src} \x1b[1;35m->\x1b[0m {tgt}\x1b[1;35m]\x1b[0m ")),
    )?;
    stdout.flush()?;
    Ok(())
}

fn move_to_bottom_and_clear() -> io::Result<u16> {
    let (_, rows) = terminal::size()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        SavePosition,
        MoveTo(0, rows.saturating_sub(1)),
        Clear(ClearType::CurrentLine),
    )?;
    stdout.flush()?;
    Ok(rows)
}

fn restore_after_cmd(rows: u16) -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        MoveTo(0, rows.saturating_sub(1)),
        Clear(ClearType::CurrentLine),
        RestorePosition,
    )?;
    stdout.flush()?;
    Ok(())
}

// ── vim-style bottom command line ─────────────────────────────────────

fn vim_command_line() -> io::Result<String> {
    let rows = move_to_bottom_and_clear()?;

    let mut stdout = io::stdout();
    execute!(stdout, Print(":"))?;
    stdout.flush()?;

    let mut cmd = String::new();
    loop {
        let ev = event::read()?;
        match ev {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                execute!(stdout, Print("\r\n"))?;
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                cmd.clear();
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            })
                if !cmd.is_empty() => {
                    cmd.pop();
                    execute!(
                        stdout,
                        MoveTo(1, rows.saturating_sub(1)),
                        Clear(ClearType::UntilNewLine),
                        Print(":"),
                        Print(&cmd),
                    )?;
                    stdout.flush()?;
                }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: mods,
                ..
            }) => {
                if c == 'u'
                    && matches!(mods, KeyModifiers::CONTROL)
                {
                    cmd.clear();
                    execute!(
                        stdout,
                        MoveTo(1, rows.saturating_sub(1)),
                        Clear(ClearType::UntilNewLine),
                        Print(":"),
                    )?;
                    stdout.flush()?;
                } else if c.is_control() {
                    // ignore other control characters
                } else {
                    cmd.push(c);
                    execute!(stdout, Print(c.to_string()))?;
                    stdout.flush()?;
                }
            }
            _ => {}
        }
    }

    restore_after_cmd(rows)?;
    Ok(cmd)
}

// ── translate input (dispatcher) ──────────────────────────────────────

fn translate_input(
    source: &str,
    target: &str,
    is_dict_mode: bool,
) -> io::Result<Option<String>> {
    if io::stdin().is_terminal() {
        raw_translate_input(source, target, is_dict_mode)
    } else {
        line_translate_input(source, target, is_dict_mode)
    }
}

fn line_translate_input(
    source: &str,
    target: &str,
    is_dict_mode: bool,
) -> io::Result<Option<String>> {
    if is_dict_mode {
        print!("[Dict: {source} -> {target}] ");
    } else {
        print!("[Translate: {source} -> {target}] ");
    }
    io::stdout().flush()?;

    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line)? == 0 {
        return Ok(None);
    }
    let line = line.trim_end().to_string();

    if let Some(cmd) = line.strip_prefix(":/") {
        Ok(Some(format!("/{cmd}")))
    } else if line.eq_ignore_ascii_case("ree") {
        Ok(Some("/reelect".into()))
    } else {
        Ok(Some(line))
    }
}

// ── raw translate input ───────────────────────────────────────────────

fn raw_translate_input(
    source: &str,
    target: &str,
    is_dict_mode: bool,
) -> io::Result<Option<String>> {
    if is_dict_mode {
        print_prompt_dict(source, target)?;
    } else {
        print_prompt(source, target)?;
    }

    let mut stdout = io::stdout();
    let mut buffer = String::new();
    let mut colon_pending = false;
    let prompt_len = if is_dict_mode {
        source.len() + target.len() + 13
    } else {
        source.len() + target.len() + 18
    };

    loop {
        let ev = event::read()?;
        let (c, mods) = match ev {
            Event::Key(KeyEvent { code: KeyCode::Char(c), modifiers: m, .. }) => (c, m),
            Event::Key(KeyEvent { code, modifiers, .. }) => {
                match (code, modifiers) {
                    (KeyCode::Enter, KeyModifiers::NONE) => {
                        execute!(stdout, Print("\r\n"))?;
                        stdout.flush()?;
                        break;
                    }
                    (KeyCode::Backspace, _) => {
                        if colon_pending {
                            colon_pending = false;
                            continue;
                        }
                        if !buffer.is_empty() {
                            buffer.pop();
                            execute!(
                                stdout,
                                MoveToColumn(prompt_len as u16),
                                Clear(ClearType::UntilNewLine),
                                Print(&buffer),
                            )?;
                            stdout.flush()?;
                        }
                        continue;
                    }
                    (KeyCode::Esc, _) => {
                        buffer.clear();
                        return Ok(None);
                    }
                    _ => continue,
                }
            }
            _ => continue,
        };

        match (mods, c) {
            (KeyModifiers::CONTROL, 'c') => {
                buffer.clear();
                execute!(stdout, Print("^C\r\n"))?;
                stdout.flush()?;
                return Ok(None);
            }
            (KeyModifiers::CONTROL, 'u') => {
                // Ctrl+U: clear line
                if colon_pending {
                    colon_pending = false;
                }
                buffer.clear();
                execute!(
                    stdout,
                    MoveToColumn(prompt_len as u16),
                    Clear(ClearType::UntilNewLine),
                )?;
                stdout.flush()?;
                continue;
            }
            _ => {
                if mods == KeyModifiers::CONTROL {
                    // already handled above
                    continue;
                }
                let c = c;
                if buffer.is_empty() && c == ':' && !colon_pending {
                    // First colon at start of input — invisible, pending state
                    colon_pending = true;
                    continue;
                }
                if colon_pending && c == '/' {
                    // :/ — enter vim command line
                    execute!(stdout, Print("\r\n"))?;
                    stdout.flush()?;
                    terminal::disable_raw_mode()?;
                    let cmd = vim_command_line()?;
                    terminal::enable_raw_mode()?;

                    let cmd = format!("/{cmd}");
                    if !cmd.is_empty() && cmd != "/" {
                        if is_dict_mode {
                            print_prompt_dict(source, target)?;
                        } else {
                            print_prompt(source, target)?;
                        }
                        return Ok(Some(cmd));
                    }
                    // empty or just "/": redraw prompt
                    buffer.clear();
                    colon_pending = false;
                    if is_dict_mode {
                        print_prompt_dict(source, target)?;
                    } else {
                        print_prompt(source, target)?;
                    }
                    continue;
                }
                if colon_pending && c == ':' {
                    // :: — treat as literal :
                    buffer.push(':');
                    buffer.push(':');
                    print!("::");
                    stdout.flush()?;
                    colon_pending = false;
                    continue;
                }
                if colon_pending {
                    // colon followed by regular char — both are literal
                    buffer.push(':');
                    print!(":");
                    colon_pending = false;
                }
                buffer.push(c);
                execute!(stdout, Print(c.to_string()))?;
                stdout.flush()?;
            }
        }
    }

    if buffer.is_empty() {
        Ok(Some(String::new()))
    } else {
        Ok(Some(buffer))
    }
}

// ── command handler in translate mode ─────────────────────────────────

enum CmdResult {
    Continue,
    Exit,
}

fn handle_translate_cmd(
    cmd: &str,
    source: &mut String,
    target: &mut String,
    _is_dict_mode: bool,
) -> CmdResult {
    let cmd = cmd.trim();

    match cmd {
        "exit" => {
            println!("Returned to ecapp main mode.");
            return CmdResult::Exit;
        }
        "reelect" | "ree" => {
            // handled specially — returns to select_language flow
            return CmdResult::Exit; // signal to re-select
        }
        "tip" => show_tip(),
        "help" | "h" => println!("{HELP_TEXT}"),
        "swap" => {
            let tmp = source.clone();
            *source = target.clone();
            *target = tmp;
            println!("Swapped: {source} -> {target}");
        }
        _ if cmd.starts_with("source ") => {
            let new_src = cmd.strip_prefix("source ").unwrap().trim().to_lowercase();
            if find_language(&new_src).is_some() {
                *source = new_src;
                println!("Source changed -> {source}");
            } else {
                println!("Unknown language '{new_src}'.");
            }
        }
        _ if cmd.starts_with("target ") => {
            let new_tgt = cmd.strip_prefix("target ").unwrap().trim().to_lowercase();
            if find_language(&new_tgt).is_some() {
                *target = new_tgt;
                println!("Target changed -> {target}");
            } else {
                println!("Unknown language '{new_tgt}'.");
            }
        }
        _ => {
            println!(
                "Unknown command. Available: exit, help/h, tip, swap, source <c>, target <c>, reelect/ree"
            );
        }
    }
    CmdResult::Continue
}

// ── parse shortcut ────────────────────────────────────────────────────

fn parse_tra_shortcut(input: &str) -> Option<(String, String)> {
    let rest = input
        .strip_prefix("translate ")
        .or_else(|| input.strip_prefix("tra "))
        .or_else(|| input.strip_prefix("tra-dir "))
        .or_else(|| input.strip_prefix("tra dir "))
        .or_else(|| input.strip_prefix("td "))?;

    let rest = rest.trim();
    let parts: Vec<&str> = rest.split('~').collect();
    if parts.len() != 2 {
        return None;
    }

    let source = parts[0].trim().trim_start_matches('/').to_lowercase();
    let target = parts[1].trim().trim_start_matches('/').to_lowercase();

    if source.is_empty() || target.is_empty() {
        return None;
    }
    Some((source, target))
}

// ── language selection (uses rustyline) ───────────────────────────────

fn select_language(rl: &mut DefaultEditor, label: &str) -> Option<String> {
    loop {
        let prompt = format!("{label} (:/tip to list, :/exit to cancel): ");
        let Ok(line) = rl.readline(&prompt) else {
            return None;
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match line {
            ":/exit" => return None,
            ":/tip" => {
                show_tip();
                continue;
            }
            _ => {
                let code = line.to_lowercase();
                if find_language(&code).is_some() {
                    return Some(code);
                }
                println!("  Invalid language code '{code}'. Use ':/tip' to see available codes.");
            }
        }
    }
}

// ── app mode ──────────────────────────────────────────────────────────

enum AppMode {
    Main,
    Translate { source: String, target: String, dict: bool },
    ApiConfig,
}

// ── main ──────────────────────────────────────────────────────────────

fn main() {
    let mut rl = DefaultEditor::new().unwrap_or_else(|e| {
        eprintln!("Failed to initialise line editor: {e}");
        std::process::exit(1);
    });

    let agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(12)))
        .timeout_connect(Some(Duration::from_secs(5)))
        .build()
        .into();

    let mut config = load_config();

    println!("Welcome to ecapp — Terminal Translation Tool");
    println!("Type 'help' or 'h' for available commands.\n");

    let mut mode = AppMode::Main;

    loop {
        match &mode {
            AppMode::Main => {
                let Ok(line) = rl.readline("[ecapp]# ") else {
                    break;
                };
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line);

                match line {
                    "help" | "h" => println!("{HELP_TEXT}"),
                    "api" => {
                        mode = AppMode::ApiConfig;
                        println!("Entering API configuration. Type 'help' for commands.\n");
                    }
                    "exit" | "quit" | "q" => {
                        println!("Goodbye!");
                        break;
                    }
                    "tra" | "translate" => {
                        println!("Setting up translation. (:/tip for codes, :/exit to cancel)");
                        let Some(source) = select_language(&mut rl, "Source language") else {
                            continue;
                        };
                        let Some(target) = select_language(&mut rl, "Target language") else {
                            continue;
                        };
                        println!("\nTranslation ready: {source} -> {target}");
                        println!("Enter text to translate. :/tip for languages, :/exit to leave.\n");
                        mode = AppMode::Translate { source, target, dict: false };
                    }
                    "tra-dir" | "tra dir" | "td" => {
                        println!(
                            "Setting up dictionary-enhanced translation. (:/tip for codes, :/exit to cancel)"
                        );
                        let Some(source) = select_language(&mut rl, "Source language") else {
                            continue;
                        };
                        let Some(target) = select_language(&mut rl, "Target language") else {
                            continue;
                        };
                        println!("\nDict mode ready: {source} -> {target}");
                        println!(
                            "Single words → dictionary (with phonetics & definitions)."
                        );
                        println!("Sentences → normal translation. :/exit to leave.\n");
                        mode = AppMode::Translate { source, target, dict: true };
                    }
                    _ if line.starts_with("tra ")
                        || line.starts_with("translate ")
                        || line.starts_with("tra-dir ")
                        || line.starts_with("tra dir ")
                        || line.starts_with("td ") =>
                    {
                        let is_dict = line.starts_with("tra-dir ")
                            || line.starts_with("tra dir ")
                            || line.starts_with("td ");

                        match parse_tra_shortcut(line) {
                            Some((src, tgt)) => {
                                if find_language(&src).is_none() {
                                    println!("Unknown source '{src}'.");
                                    continue;
                                }
                                if find_language(&tgt).is_none() {
                                    println!("Unknown target '{tgt}'.");
                                    continue;
                                }
                                if is_dict {
                                    println!("Dict mode ready: {src} -> {tgt}");
                                    println!("Single words → dictionary. Sentences → translation.\n");
                                } else {
                                    println!("Translation ready: {src} -> {tgt}");
                                    println!("Enter text to translate.\n");
                                }
                                mode = AppMode::Translate { source: src, target: tgt, dict: is_dict };
                            }
                            None => {
                                println!(
                                    "Usage: tra /<src> ~ <tgt>   (or tra-dir /<src> ~ <tgt>)"
                                );
                            }
                        }
                    }
                    _ => {
                        println!("Unknown command '{line}'. Type 'help' for available commands.");
                    }
                }
            }

            AppMode::Translate { source, target, dict: is_dict } => {
                let mut src = source.clone();
                let mut tgt = target.clone();
                let is_dict = *is_dict;

                let is_tty = io::stdin().is_terminal();
                if is_tty {
                    let _ = terminal::enable_raw_mode();
                }
                let result = translate_input(&src, &tgt, is_dict);
                if is_tty {
                    let _ = terminal::disable_raw_mode();
                }

                match result {
                    Ok(Some(input)) if input.is_empty() => {
                        mode = AppMode::Translate { source: src, target: tgt, dict: is_dict };
                        continue;
                    }
                    Ok(Some(input)) => {
                        if let Some(cmd) = input.strip_prefix("/") {
                            if cmd == "reelect" || cmd == "ree" {
                                println!("\r\nRe-selecting languages…");
                                let Some(new_src) =
                                    select_language(&mut rl, "Source language")
                                else {
                                    mode = AppMode::Main;
                                    continue;
                                };
                                let Some(new_tgt) =
                                    select_language(&mut rl, "Target language")
                                else {
                                    mode = AppMode::Main;
                                    continue;
                                };
                                println!("\nReady: {new_src} -> {new_tgt}\n");
                                mode = AppMode::Translate {
                                    source: new_src,
                                    target: new_tgt,
                                    dict: is_dict,
                                };
                                continue;
                            }
                            if cmd == "exit" {
                                mode = AppMode::Main;
                                println!("Returned to ecapp main mode.");
                                continue;
                            }
                            match handle_translate_cmd(cmd, &mut src, &mut tgt, is_dict) {
                                CmdResult::Exit => {
                                    if cmd == "reelect" || cmd == "ree" {
                                        let Some(new_src) =
                                            select_language(&mut rl, "Source language")
                                        else {
                                            mode = AppMode::Main;
                                            continue;
                                        };
                                        let Some(new_tgt) =
                                            select_language(&mut rl, "Target language")
                                        else {
                                            mode = AppMode::Main;
                                            continue;
                                        };
                                        println!("\nReady: {new_src} -> {new_tgt}\n");
                                        mode = AppMode::Translate {
                                            source: new_src,
                                            target: new_tgt,
                                            dict: is_dict,
                                        };
                                    } else {
                                        mode = AppMode::Main;
                                    }
                                }
                                CmdResult::Continue => {
                                    mode = AppMode::Translate {
                                        source: src.clone(),
                                        target: tgt.clone(),
                                        dict: is_dict,
                                    };
                                }
                            }
                            continue;
                        }

                        if is_dict && is_single_word(&input) {
                            println!();
                            let trimmed = input.trim();
                            match lookup_dictionary(&agent, trimmed, &src) {
                                Ok(entries) => {
                                    let formatted = format_dictionary(&entries);
                                    println!("{formatted}");
                                }
                                Err(_e) => {
                                    match translate_dispatch(&agent, &config, trimmed, &src, &tgt) {
                                        Ok(t) => println!("\x1b[32m{t}\x1b[0m\n"),
                                        Err(e) => eprintln!("Translation failed: {e}\n"),
                                    }
                                }
                            }
                        } else {
                            match translate_dispatch(&agent, &config, &input, &src, &tgt) {
                                Ok(translated) => println!("\x1b[32m{translated}\x1b[0m\n"),
                                Err(e) => eprintln!("Translation failed: {e}\n"),
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!("Input error: {e}");
                        break;
                    }
                }

                mode = AppMode::Translate { source: src, target: tgt, dict: is_dict };
            }

            AppMode::ApiConfig => {
                let backend_name = BACKENDS
                    .iter()
                    .find(|(id, _, _)| *id == config.backend)
                    .map(|(_, name, _)| *name)
                    .unwrap_or("Unknown");

                let prompt = format!("[api: {backend_name}]# ");
                let Ok(line) = rl.readline(&prompt) else {
                    mode = AppMode::Main;
                    continue;
                };
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line);

                match line {
                    "help" | "h" => {
                        println!();
                        println!("  API Configuration Commands:");
                        println!("    show                 Display current config");
                        println!("    set <backend>        Switch backend (mymemory / google / deepl)");
                        println!("    key <api-key>        Set API key for current backend");
                        println!("    key                  Clear the API key");
                        println!("    exit                 Return to main mode");
                        println!();
                    }
                    "show" => {
                        println!();
                        println!("  Current backend: {}", config.backend);
                        for (id, name, desc) in BACKENDS {
                            let active = if *id == config.backend { " [ACTIVE]" } else { "" };
                            println!("    {id:<12} {name:<20} {desc}{active}");
                        }
                        if config.api_key.as_deref().is_some_and(|k| !k.is_empty()) {
                            let masked = mask_key(config.api_key.as_deref().unwrap_or(""));
                            println!("  API key: {masked}");
                        } else {
                            println!("  API key: (not set)");
                        }
                        println!();
                    }
                    "exit" => {
                        save_config(&config);
                        mode = AppMode::Main;
                        println!("API config saved.");
                    }
                    _ if line.starts_with("set ") => {
                        let backend = line.strip_prefix("set ").unwrap().trim().to_lowercase();
                        if BACKENDS.iter().any(|(id, _, _)| *id == backend) {
                            config.backend = backend;
                            config.api_key = None;
                            save_config(&config);
                            let name = BACKENDS
                                .iter()
                                .find(|(id, _, _)| *id == config.backend)
                                .map(|(_, name, _)| *name)
                                .unwrap_or("");
                            println!("Switched to {name}. Remember to set an API key with 'key'.");
                        } else {
                            println!("Unknown backend '{backend}'. Available: mymemory, google, deepl");
                        }
                    }
                    _ if line.starts_with("key ") => {
                        let key = line.strip_prefix("key ").unwrap().trim();
                        if key.is_empty() {
                            config.api_key = None;
                            println!("API key cleared.");
                        } else {
                            config.api_key = Some(key.to_string());
                            let masked = mask_key(key);
                            println!("API key set: {masked}");
                        }
                        save_config(&config);
                    }
                    "key" => {
                        config.api_key = None;
                        save_config(&config);
                        println!("API key cleared.");
                    }
                    _ => {
                        println!("Unknown command. Try 'help'.");
                    }
                }
            }
        }
    }

    let _ = terminal::disable_raw_mode();
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        "***".into()
    } else {
        format!("{}...{}", &key[..4], &key[key.len()-4..])
    }
}
