//! Deterministic mock provider for debug builds.
//!
//! This module is never compiled in release builds. Output is labelled
//! `mock`, and the UI keeps a persistent MOCK banner on screen.

use std::time::Duration;

use magi_core::{
    DebateCall, DebateReply, Opening, OpeningCall, PersonaId, Result, Stance, VoteCall, VoteCast,
};

use super::MagiProvider;
use super::parsing::{parse_debate_reply, parse_opening, parse_vote};

pub struct DemoProvider;

impl Default for DemoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoProvider {
    pub fn new() -> Self {
        Self
    }
}

fn hash(text: &str) -> u64 {
    let mut value: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        value ^= u64::from(*byte);
        value = value.wrapping_mul(0x0000_0100_0000_01b3);
    }
    value
}

fn persona_index(persona: PersonaId) -> usize {
    match persona {
        PersonaId::Melchior => 0,
        PersonaId::Balthasar => 1,
        PersonaId::Casper => 2,
    }
}

fn opening_stance(question: &str, persona: PersonaId) -> Stance {
    let table = match hash(question) % 6 {
        0 => [Stance::Approve, Stance::Approve, Stance::Approve],
        1 => [Stance::Reject, Stance::Reject, Stance::Reject],
        2 => [Stance::Approve, Stance::Approve, Stance::Reject],
        3 => [Stance::Reject, Stance::Reject, Stance::Approve],
        4 => [Stance::Approve, Stance::Reject, Stance::Abstain],
        _ => [Stance::Abstain, Stance::Abstain, Stance::Approve],
    };
    table[persona_index(persona)]
}

fn final_stance(question: &str, persona: PersonaId) -> Stance {
    let opening = opening_stance(question, persona);

    // Occasionally let CASPER change its mind so the UI can demonstrate the
    // CHANGED marker.
    if persona == PersonaId::Casper && hash(question).is_multiple_of(5) {
        match opening {
            Stance::Approve => Stance::Reject,
            Stance::Reject => Stance::Approve,
            Stance::Abstain => Stance::Approve,
        }
    } else {
        opening
    }
}

fn confidence(question: &str, persona: PersonaId, salt: &str) -> u8 {
    let seed = hash(&format!("{question}:{}:{salt}", persona.as_str()));
    55 + (seed % 31) as u8
}

fn summary(persona: PersonaId, stance: Stance) -> &'static str {
    match (persona, stance) {
        (PersonaId::Melchior, Stance::Approve) => {
            "从证据与推理链看，主张与现有事实一致，关键假设可检验，收益大于可识别的风险。"
        }
        (PersonaId::Melchior, Stance::Reject) => {
            "核心主张依赖未经检验的假设，现有证据不足以支持该结论，风险被明显低估。"
        }
        (PersonaId::Melchior, Stance::Abstain) => {
            "关键事实缺失，证据链不足以作出可靠判断；在补充数据之前应保持保留。"
        }
        (PersonaId::Balthasar, Stance::Approve) => {
            "该选择可逆、下行风险可控，对相关人的长期保护优于维持现状。"
        }
        (PersonaId::Balthasar, Stance::Reject) => {
            "潜在损害不易恢复，受影响最深的人缺少保护，当前方案的安全边际不足。"
        }
        (PersonaId::Balthasar, Stance::Abstain) => {
            "现有信息无法排除对脆弱方的重大影响，需要让更多人参与判断。"
        }
        (PersonaId::Casper, Stance::Approve) => {
            "从信任、关系与感受看，这个方向更接近人们真实的期待，沟通成本可控。"
        }
        (PersonaId::Casper, Stance::Reject) => {
            "方案忽略了人的情绪与尊严，可能侵蚀信任，造成难以修复的关系裂痕。"
        }
        (PersonaId::Casper, Stance::Abstain) => {
            "相关人的真实感受与处境还不够清楚，直接表决容易替他人做决定。"
        }
    }
}

fn key_points(persona: PersonaId) -> Vec<&'static str> {
    match persona {
        PersonaId::Melchior => vec!["论证强度应与结论强度匹配", "明确什么证据会推翻当前判断"],
        PersonaId::Balthasar => vec![
            "优先检查是否存在不可逆的下行风险",
            "确认受影响最大的人是否有发言权",
        ],
        PersonaId::Casper => vec!["注意未被说出口的情绪与权力关系", "把直觉当作待验证的假设"],
    }
}

fn risks(persona: PersonaId) -> Vec<&'static str> {
    match persona {
        PersonaId::Melchior => vec!["关键假设尚未被独立验证"],
        PersonaId::Balthasar => vec!["最坏情况下可能触及不可恢复的损害"],
        PersonaId::Casper => vec!["可能忽视沉默相关方的真实感受"],
    }
}

fn opening_json(question: &str, persona: PersonaId) -> String {
    let stance = opening_stance(question, persona);
    serde_json::json!({
        "stance": stance,
        "confidence": confidence(question, persona, "opening"),
        "summary": summary(persona, stance),
        "key_points": key_points(persona),
        "risks": risks(persona),
    })
    .to_string()
}

fn debate_text(question: &str, persona: PersonaId) -> String {
    let stance = opening_stance(question, persona);
    let stance_cn = match stance {
        Stance::Approve => "支持",
        Stance::Reject => "反对",
        Stance::Abstain => "保留",
    };
    let peers = match persona {
        PersonaId::Melchior => "BALTHASAR 与 CASPER",
        PersonaId::Balthasar => "MELCHIOR 与 CASPER",
        PersonaId::Casper => "MELCHIOR 与 BALTHASAR",
    };

    let lens = match persona {
        PersonaId::Melchior => {
            "对我而言，关键是证据是否可检验、结论是否与数据同强度。如果拿不出可复核的证据，更强的信心只会放大错误。"
        }
        PersonaId::Balthasar => {
            "我关心的是最坏后果是否可逆，以及最脆弱的一方是否被看见。收益再大，也不能把不可恢复的代价转嫁给没有选择权的人。"
        }
        PersonaId::Casper => {
            "我关心的是人们是否被尊重，以及这个决定会如何改变彼此之间的信任。很多真实感受不会出现在数据里，但会在关系里留下痕迹。"
        }
    };

    format!(
        "我仍然倾向于{stance_cn}，但{peers}提出的风险值得正面回应。{lens}\
         我们的分歧不在于事实本身，而在于对不确定性的容忍度。如果对方能给出可复核的证据，\
         或者说明如何为最坏情况兜底，我愿意修正判断；否则我建议按更保守的路径推进。"
    )
}

fn vote_json(question: &str, persona: PersonaId) -> String {
    let stance = final_stance(question, persona);
    let reason = match (persona, stance) {
        (PersonaId::Melchior, _) => "综合辩论中的证据与反例，当前结论的置信度仍高于我的最低标准。",
        (PersonaId::Balthasar, _) => "在确认下行风险可以被兜底之前，我倾向于更保守的选项。",
        (PersonaId::Casper, _) => "我听到的顾虑没有被充分回应，人的感受不能只用效率来衡量。",
    };

    serde_json::json!({
        "stance": stance,
        "confidence": confidence(question, persona, "vote"),
        "reason": reason,
    })
    .to_string()
}

async fn stream_text(text: &str, on_delta: &mut (dyn FnMut(String) + Send)) {
    let chars: Vec<char> = text.chars().collect();
    for chunk in chars.chunks(12) {
        let piece: String = chunk.iter().collect();
        on_delta(piece);
        tokio::time::sleep(Duration::from_millis(12)).await;
    }
}

impl MagiProvider for DemoProvider {
    async fn opening_stream(
        &self,
        call: &OpeningCall,
        on_delta: &mut (dyn FnMut(String) + Send),
    ) -> Result<Opening> {
        let text = opening_json(&call.question, call.persona);
        stream_text(&text, on_delta).await;
        parse_opening(&text, call.persona, "mock")
    }

    async fn debate_stream(
        &self,
        call: &DebateCall,
        on_delta: &mut (dyn FnMut(String) + Send),
    ) -> Result<DebateReply> {
        let text = debate_text(&call.question, call.persona);
        stream_text(&text, on_delta).await;
        parse_debate_reply(&text, call.persona, "mock")
    }

    async fn cast_vote(&self, call: &VoteCall) -> Result<VoteCast> {
        tokio::time::sleep(Duration::from_millis(250)).await;
        parse_vote(
            &vote_json(&call.question, call.persona),
            call.persona,
            "mock",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_the_same_question() {
        let question = "Should we ship on Friday?";
        assert_eq!(
            opening_json(question, PersonaId::Melchior),
            opening_json(question, PersonaId::Melchior)
        );
    }

    #[test]
    fn covers_all_stances() {
        let mut seen = [false; 3];
        for index in 0..200 {
            let question = format!("question {index}");
            let stance = opening_stance(&question, PersonaId::Melchior);
            let slot = match stance {
                Stance::Approve => 0,
                Stance::Reject => 1,
                Stance::Abstain => 2,
            };
            seen[slot] = true;
        }
        assert!(seen.iter().all(|value| *value));
    }
}
