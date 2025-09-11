use std::{borrow::Cow, fmt::Display, hash::{Hash, Hasher}, sync::LazyLock};

#[derive(Debug, Default)]
#[allow(clippy::manual_non_exhaustive)] 
pub struct Character {
    pub jpspeaker: &'static str,
    pub jpshort: &'static str,
    pub enspeaker: &'static str,
    pub gender: &'static str,
    pub aliases: Box<[(&'static str, &'static str)]>,
    _private: ()
}

impl Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name: {} ({}) | Gender: {}",
            self.enspeaker,
            self.jpspeaker,
            self.gender
        )?;

        if !self.aliases.is_empty() {
            f.write_str(" | Aliases: ")?;
            let mut aliases = self.aliases.iter().copied().peekable();
            while let Some((jp, en)) = aliases.next() {
                write!(f, "{en} ({jp})")?;
                if aliases.peek().is_some() {
                    f.write_str(", ")?;
                }
            }
        }

        Ok(())
    }
}

impl PartialEq for Character {
    fn eq(&self, other: &Self) -> bool {
        self.jpspeaker.eq(other.jpspeaker)
    }
}

impl Eq for Character {}

impl Hash for Character {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.jpspeaker.hash(state)
    }
}

pub static CHARACTERS: LazyLock<Box<[Character]>> = LazyLock::new(|| Box::new([
    Character { jpspeaker: "少女", enspeaker: "Girl", gender: "Female", ..Default::default() },
    Character { jpspeaker: "魔法使い", enspeaker: "Wizard", gender: "Male", ..Default::default() },
    Character { jpspeaker: "玻ヰ璃[ハイリ]＝ラリック", jpshort: "玻ヰ璃", enspeaker: "Hairi Lalique", gender: "Female", ..Default::default() },
    Character { jpspeaker: "カンパネラ", enspeaker: "Campanella", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "歌紫歌[カシカ]＝ガレ", jpshort: "歌紫歌", enspeaker: "Kashika Galle", gender: "Male", ..Default::default() },
    Character { jpspeaker: "糸遠[シオン]＝ラリック", jpshort: "糸遠", enspeaker: "Shion Lalique", gender: "Male", ..Default::default() },
    Character { jpspeaker: "衿栖[エリス]＝シュナイダー", jpshort: "衿栖", enspeaker: "Eris Schneider", gender: "Female", ..Default::default() },
    Character { jpspeaker: "綸燈[リンドウ]＝ウェステリア", jpshort: "綸燈", enspeaker: "Rindo Westeria", gender: "Male", ..Default::default() },
    Character { jpspeaker: "泣虎[ナトラ]＝ピオニー", jpshort: "泣虎", enspeaker: "Natra Peony", gender: "Male", ..Default::default() },
    Character { jpspeaker: "廻螺[エラ]＝アマルリック", jpshort: "廻螺", enspeaker: "Ela Amalric", gender: "Male", ..Default::default() },
    Character { jpspeaker: "憂漣[ユーレン]＝ミュラー", jpshort: "憂漣", enspeaker: "Ulen Mueller", gender: "Male", ..Default::default() },
    Character { jpspeaker: "紫鳶[シエン]＝クリノクロア", jpshort: "紫鳶", enspeaker: "Shien Clinochlore", gender: "Male", ..Default::default() },
    Character { jpspeaker: "黒禰[クロネ]＝スピネル", jpshort: "黒禰", enspeaker: "Klone Spinel", gender: "Male", ..Default::default() },
    Character { jpspeaker: "番人１", enspeaker: "Guard 1", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "男１", enspeaker: "Man 1", gender: "Male", ..Default::default() },
    Character { jpspeaker: "二人", enspeaker: "Two People", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "Ｍ", enspeaker: "M", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "番人２", enspeaker: "Guard 2", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "女性Ａ", enspeaker: "Woman A", gender: "Female", ..Default::default() },
    Character { jpspeaker: "男性Ａ", enspeaker: "Man A", gender: "Male", ..Default::default() },
    Character { jpspeaker: "子供Ａ", enspeaker: "Child A", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "紫鳶＆黒禰", enspeaker: "Shien & Klone", gender: "Male", ..Default::default() },
    Character { jpspeaker: "猿", enspeaker: "Monkey", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "憂漣[ユーレン]ミュラー", enspeaker: "Ulen Mueller", gender: "Male", ..Default::default() },
    Character { jpspeaker: "ハルモニア", enspeaker: "Harmonia", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "憂漣[ユーレン]=ミュラー", enspeaker: "Ulen Mueller", gender: "Male", ..Default::default() },
    Character { jpspeaker: "瑪衣[メイ]", jpshort: "瑪衣", enspeaker: "Mei", gender: "Female", ..Default::default() },
    Character { jpspeaker: "瑪衣[メイ] ", enspeaker: "Mei", gender: "Female", ..Default::default() },
    Character { jpspeaker: "司書", enspeaker: "Librarian", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "研究者Ａ", enspeaker: "Researcher A", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "助手", enspeaker: "Assistant", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "研究者Ｂ", enspeaker: "Researcher B", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "アロマ店店主", enspeaker: "Aroma Shop Owner", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "番人Ａ", enspeaker: "Guard A", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "番人Ｂ", enspeaker: "Guard B", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "住民Ａ", enspeaker: "Resident A", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "住民Ｂ", enspeaker: "Resident B", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "刈鐘[カリガネ]", jpshort: "刈鐘", enspeaker: "Karigane", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "三人", enspeaker: "Three People", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "女性記者Ａ", enspeaker: "Female Reporter A", gender: "Female", ..Default::default() },
    Character { jpspeaker: "晩歌[バンカ]", jpshort: "晩歌", enspeaker: "Banka", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "少女Ａ", enspeaker: "Girl A", gender: "Female", ..Default::default() },
    Character { jpspeaker: "男性", enspeaker: "Man", gender: "Male", ..Default::default() },
    Character { jpspeaker: "店員", enspeaker: "Shop Clerk", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "霞[カスミ]", jpshort: "霞", enspeaker: "Kasumi", gender: "Female", ..Default::default() },
    Character { jpspeaker: "霞[カスミ]　　", enspeaker: "Kasumi", gender: "Female", ..Default::default() },
    Character { jpspeaker: "おばあさん", enspeaker: "Grandmother", gender: "Female", ..Default::default() },
    Character { jpspeaker: "子供", enspeaker: "Child", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "おばあちゃん", enspeaker: "Grandma", gender: "Female", ..Default::default() },
    Character { jpspeaker: "少年", enspeaker: "Boy", gender: "Male", ..Default::default() },
    Character { jpspeaker: "泣虎[ナトラ]＝ピオニー　", enspeaker: "Natra Peony", gender: "Male", ..Default::default() },
    Character { jpspeaker: "女性１", enspeaker: "Woman 1", gender: "Female", ..Default::default() },
    Character { jpspeaker: "女性２", enspeaker: "Woman 2", gender: "Female", ..Default::default() },
    Character { jpspeaker: "門番", enspeaker: "Gatekeeper", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "歌紫歌＆糸遠", enspeaker: "Kashika & Shion", gender: "Male", ..Default::default() },
    Character { jpspeaker: "歌紫歌", enspeaker: "Kashika", gender: "Male", ..Default::default() },
    Character { jpspeaker: "男性１", enspeaker: "Man 1", gender: "Male", ..Default::default() },
    Character { jpspeaker: "初代Ｍ", enspeaker: "First M", gender: "Unknown", ..Default::default() },
    Character { jpspeaker: "王", enspeaker: "King", gender: "Male", ..Default::default() },
    Character { jpspeaker: "瑠璃[ルリ]", jpshort: "瑠璃", enspeaker: "Ruri", gender: "Female", ..Default::default() },
]));

#[derive(Clone, Debug)]
pub enum EnSpeaker {
    Str(Cow<'static, str>),
    Character(&'static Character)
}

impl Display for EnSpeaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnSpeaker::Str(s) => Display::fmt(s, f),
            EnSpeaker::Character(c) => Display::fmt(c.enspeaker, f)
        }
    }
}

pub fn decode_jp_speaker(jpspeaker: &str) -> anyhow::Result<EnSpeaker> {
    if jpspeaker == "？？？" {
        return Ok(EnSpeaker::Str("???".into()));
    }
    for char in CHARACTERS.iter() {
        if char.jpspeaker == jpspeaker {
            return Ok(EnSpeaker::Character(char));
        }

        if jpspeaker.strip_prefix(char.jpspeaker).is_some_and(|s| s == "の声") {
            return Ok(EnSpeaker::Str((char.enspeaker.to_owned() + "'s voice").into()));
        }
    }
    Err(anyhow::anyhow!("bro I don't know {jpspeaker}"))
}

pub static ELEMENTS: &[(&str, &str)] = &[
    ("透京", "[element] Name: Tokyo (透京) | Type: Place"),
    ("透境門", "[element] Name: Tokyomon (透境門) | Type: Place"),
    ("透迷ノ園", "[element] Name: Tomei-no-sono (透迷ノ園) | Type: Place"),
    ("透淵ノ森", "[element] Name: Toen-no-mori (透淵ノ森) | Type: Place"),
    ("白鴉", "[element] Name: Hakua (白鴉) | Type: Familiar"), // adjust the type on this one tbh
    ("黒死紋事件", "[element] Name: Black Death Mark Incident (黒死紋事件) | Type: Event"), // copilot suggestion
    ("時輪のアストロラビ", "[element] Name: Astronomical Clock (時輪のアストロラビ) | Type: Equipment") // pulled this one out of my ass
];