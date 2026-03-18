use crate::tokens::TokenRole;

#[derive(Debug, Clone, Copy)]
pub struct PreviewSegment {
    pub text: &'static str,
    pub role: Option<TokenRole>,
}

pub fn sample_code() -> Vec<Vec<PreviewSegment>> {
    use TokenRole::*;

    vec![
        vec![
            seg("fn", Some(Keyword)),
            seg(" ", None),
            seg("build_theme", Some(Function)),
            seg("()", None),
            seg(" -> ", None),
            seg("ResolvedTheme", Some(Type)),
            seg(" {", None),
        ],
        vec![
            seg("    let ", Some(Keyword)),
            seg("background", Some(Variable)),
            seg(" = ", None),
            seg("\"#0D1017\"", Some(String)),
            seg(";", None),
        ],
        vec![
            seg("    let ", Some(Keyword)),
            seg("selection_mix", Some(Variable)),
            seg(" = ", None),
            seg("0.35", Some(Number)),
            seg(";", None),
        ],
        vec![seg("    // Blend accent with the surface", Some(Comment))],
        vec![
            seg("    ", None),
            seg("theme", Some(Variable)),
            seg(".", None),
            seg("resolve", Some(Function)),
            seg("(", None),
            seg("selection_mix", Some(Variable)),
            seg(")", None),
        ],
        vec![seg("}", None)],
    ]
}

const fn seg(text: &'static str, role: Option<TokenRole>) -> PreviewSegment {
    PreviewSegment { text, role }
}
