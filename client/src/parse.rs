use wish_shared::{Participant, Slot};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ParseWarning {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ParsedParticipant {
    pub mail: String,
    pub wish: Vec<i32>,
}

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub slots: Vec<Slot>,
    pub participants: Vec<ParsedParticipant>,
    pub errors: Vec<ParseError>,
    pub warnings: Vec<ParseWarning>,
}

/// Parse the admin text editor format into slots and participants.
///
/// Format:
/// ```text
/// [slots]
/// "Slot Name" min max
/// ...
/// [participants]
/// "email@example.com" 0 1 2 ...
/// ```
///
/// Lines starting with % or # are comments.
pub fn parse(text: &str) -> ParseResult {
    #[derive(Clone, Copy)]
    enum Section {
        Before,
        Slots,
        Participants,
    }

    fn parse_u32(token: &str, line_idx: usize, errors: &mut Vec<ParseError>) -> Option<u32> {
        match token.parse() {
            Ok(v) => Some(v),
            Err(_) => {
                errors.push(ParseError {
                    line: line_idx,
                    message: format!("'{token}' is not a valid number"),
                });
                None
            }
        }
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut slots: Vec<Slot> = Vec::new();
    let mut participants: Vec<ParsedParticipant> = Vec::new();
    let mut section = Section::Before;
    let mut sum_vmin = 0u32;
    let mut sum_vmax = 0u32;

    for (line_idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('%') || trimmed.starts_with('#') {
            continue;
        }

        // Section headers
        if trimmed.starts_with('[') {
            if let Some(end) = trimmed.find(']') {
                let word = trimmed[1..end].trim();
                match (section, word) {
                    (Section::Before, "slots") => section = Section::Slots,
                    (Section::Slots, "participants") => section = Section::Participants,
                    (Section::Before, _) => {
                        errors.push(ParseError {
                            line: line_idx,
                            message: "Expected [slots] section first".to_string(),
                        });
                        return ParseResult {
                            slots,
                            participants,
                            errors,
                            warnings,
                        };
                    }
                    (Section::Slots, _) => {
                        errors.push(ParseError {
                            line: line_idx,
                            message: "Expected [participants] section".to_string(),
                        });
                        return ParseResult {
                            slots,
                            participants,
                            errors,
                            warnings,
                        };
                    }
                    (Section::Participants, _) => {
                        errors.push(ParseError {
                            line: line_idx,
                            message: "Unexpected section".to_string(),
                        });
                        return ParseResult {
                            slots,
                            participants,
                            errors,
                            warnings,
                        };
                    }
                }
            } else {
                errors.push(ParseError {
                    line: line_idx,
                    message: "Unclosed section bracket".to_string(),
                });
                return ParseResult {
                    slots,
                    participants,
                    errors,
                    warnings,
                };
            }
            continue;
        }

        if matches!(section, Section::Before) {
            warnings.push(ParseWarning {
                line: line_idx,
                message: "Line ignored: not in a section. [slots] missing?".to_string(),
            });
            continue;
        }

        // Parse data row: starts with a quoted string
        if !trimmed.starts_with('"') {
            errors.push(ParseError {
                line: line_idx,
                message: "Expected a quoted string".to_string(),
            });
            continue;
        }

        let (string, rest) = match parse_quoted_string(trimmed) {
            Some(r) => r,
            None => {
                errors.push(ParseError {
                    line: line_idx,
                    message: "Invalid quoted string".to_string(),
                });
                continue;
            }
        };

        // Strip trailing comment
        let rest = if let Some(idx) = rest.find('%').or_else(|| rest.find('#')) {
            &rest[..idx]
        } else {
            rest
        };

        let tokens: Vec<&str> = rest.split_whitespace().collect();

        match section {
            Section::Before => unreachable!(),
            Section::Slots => {
                if tokens.len() < 2 {
                    errors.push(ParseError {
                        line: line_idx,
                        message: "Expected: \"name\" min max".to_string(),
                    });
                    continue;
                }
                let vmin = match parse_u32(tokens[0], line_idx, &mut errors) {
                    Some(v) => v,
                    None => continue,
                };
                let vmax = match parse_u32(tokens[1], line_idx, &mut errors) {
                    Some(v) => v,
                    None => continue,
                };
                if vmax < vmin {
                    errors.push(ParseError {
                        line: line_idx,
                        message: "max must be >= min".to_string(),
                    });
                    continue;
                }
                sum_vmin += vmin;
                sum_vmax += vmax;
                slots.push(Slot {
                    name: string.to_string(),
                    vmin,
                    vmax,
                });
            }
            Section::Participants => {
                let mut wish = Vec::new();
                for tok in &tokens {
                    match tok.parse::<i32>() {
                        Ok(v) if v >= 0 => wish.push(v),
                        Ok(_) => {
                            errors.push(ParseError {
                                line: line_idx,
                                message: format!("'{tok}' is not a non-negative number"),
                            });
                        }
                        Err(_) => {
                            errors.push(ParseError {
                                line: line_idx,
                                message: format!("'{tok}' is not a valid number"),
                            });
                        }
                    }
                }

                // Check duplicate mails
                if participants.iter().any(|p| p.mail == string) {
                    errors.push(ParseError {
                        line: line_idx,
                        message: "This email appears multiple times".to_string(),
                    });
                }

                if wish.len() != slots.len() {
                    errors.push(ParseError {
                        line: line_idx,
                        message: format!(
                            "Expected {} wish values (one per slot), got {}",
                            slots.len(),
                            wish.len()
                        ),
                    });
                }

                if participants.len() as u32 + 1 > sum_vmax {
                    errors.push(ParseError {
                        line: line_idx,
                        message: "Too many participants for the maximal bounds".to_string(),
                    });
                }

                // Fairness check
                if errors.is_empty() && wish.len() == slots.len() {
                    let mut sorted = wish.clone();
                    sorted.sort();
                    for (i, &v) in sorted.iter().enumerate() {
                        if v > i as i32 {
                            warnings.push(ParseWarning {
                                line: line_idx,
                                message: "This wish is not fair".to_string(),
                            });
                            break;
                        }
                    }
                }

                participants.push(ParsedParticipant {
                    mail: string.to_string(),
                    wish,
                });
            }
        }
    }

    if errors.is_empty() && (participants.len() as u32) < sum_vmin {
        errors.push(ParseError {
            line: text.lines().count(),
            message: "Not enough participants for the minimal bounds".to_string(),
        });
    }

    ParseResult {
        slots,
        participants,
        errors,
        warnings,
    }
}

pub fn parse_quoted_string(s: &str) -> Option<(&str, &str)> {
    if !s.starts_with('"') {
        return None;
    }
    let rest = &s[1..];
    let end = rest.find('"')?;
    Some((&rest[..end], rest[end + 1..].trim()))
}

/// Convert admin data back into the text editor format.
/// Participants with an empty `id` render without the `% wish?{id}` marker.
pub fn to_editor_text(slots: &[Slot], participants: &[Participant]) -> String {
    let mut text = String::from("[slots]\n");

    // Calculate column widths for alignment
    let slot_names: Vec<String> = slots.iter().map(|s| format!("\"{}\"", s.name)).collect();
    let max_name_len = slot_names.iter().map(|n| n.len()).max().unwrap_or(0);

    for (i, slot) in slots.iter().enumerate() {
        let name = &slot_names[i];
        let padding = " ".repeat(max_name_len - name.len() + 1);
        text.push_str(&format!(
            "{name}{padding}{} {}  % slot #{}\n",
            slot.vmin,
            slot.vmax,
            i + 1
        ));
    }

    text.push_str("\n[participants]\n");

    // Header comment
    text.push_str("% slots:");
    for i in 0..slots.len() {
        text.push_str(&format!(" #{}", i + 1));
    }
    text.push('\n');

    let mail_names: Vec<String> = participants
        .iter()
        .map(|p| format!("\"{}\"", p.mail))
        .collect();
    let max_mail_len = mail_names.iter().map(|n| n.len()).max().unwrap_or(0);

    for (i, p) in participants.iter().enumerate() {
        let name = &mail_names[i];
        let padding = " ".repeat(max_mail_len - name.len() + 1);
        text.push_str(name);
        text.push_str(&padding);
        for (j, w) in p.wish.iter().enumerate() {
            text.push_str(&w.to_string());
            if j < p.wish.len() - 1 {
                text.push(' ');
            }
        }
        text.push_str(&format!("  % {}", p.status.label()));
        if !p.id.is_empty() {
            text.push_str(&format!("  % wish?{}", p.id));
        }
        text.push('\n');
    }

    text
}

/// Format the results of the assignment as text (statistics + results).
pub fn format_results(
    slots: &[Slot],
    participants: &[(String, Vec<i32>)],
    result: &[usize],
) -> String {
    let mut text = String::new();

    // Total score
    let score: i64 = participants
        .iter()
        .zip(result.iter())
        .map(|((_, wish), &slot)| {
            let w = wish[slot] as i64;
            w * w
        })
        .sum();

    text.push_str("[statistics]\n");
    text.push_str(&format!("\"total score\" {score}\n\n"));

    // Per-slot stats
    let max_wish = participants
        .iter()
        .flat_map(|(_, w)| w.iter())
        .cloned()
        .max()
        .unwrap_or(0) as usize;

    text.push_str(&format!("{:<30} {:>5}", "% slot name", "# ptc"));
    for w in 0..=max_wish {
        text.push_str(&format!(" {:>5}", format!("w={w}")));
    }
    text.push('\n');

    let mut total_choices = vec![0usize; max_wish + 1];
    for (i, slot) in slots.iter().enumerate() {
        let mut slot_choices = vec![0usize; max_wish + 1];
        let mut count = 0;
        for (pi, &si) in result.iter().enumerate() {
            if si == i {
                let w = participants[pi].1[i] as usize;
                if w <= max_wish {
                    slot_choices[w] += 1;
                    total_choices[w] += 1;
                }
                count += 1;
            }
        }
        text.push_str(&format!(
            "{:<30} {:>5}",
            format!("\"{}\"", slot.name),
            count
        ));
        for c in &slot_choices {
            text.push_str(&format!(" {:>5}", c));
        }
        text.push('\n');
    }

    // Totals
    text.push_str(&format!("{:<30} {:>5}", "% ===", "==="));
    for _ in 0..=max_wish {
        text.push_str("   ===");
    }
    text.push('\n');
    text.push_str(&format!("{:<30} {:>5}", "\"total\"", participants.len()));
    for c in &total_choices {
        text.push_str(&format!(" {:>5}", c));
    }
    text.push('\n');

    // Per-participant results
    text.push_str("\n[results]\n");
    text.push_str(&format!(
        "{:<35} {:<30} {:>5} {:>5}\n",
        "% mail", "slot name", "slot#", "wish"
    ));
    for (pi, &si) in result.iter().enumerate() {
        text.push_str(&format!(
            "{:<35} {:<30} {:>5} {:>5}\n",
            format!("\"{}\"", participants[pi].0),
            format!("\"{}\"", slots[si].name),
            si + 1,
            participants[pi].1[si],
        ));
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use wish_shared::ParticipantStatus;

    // ── parse() basic ──────────────────────────────────────────────

    #[test]
    fn parse_valid_minimal() {
        let text = "[slots]\n\"A\" 1 2\n\"B\" 1 2\n\n[participants]\n\"x@a\" 0 1\n\"y@b\" 1 0\n";
        let r = parse(text);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
        assert_eq!(r.slots.len(), 2);
        assert_eq!(r.slots[0].name, "A");
        assert_eq!(r.slots[0].vmin, 1);
        assert_eq!(r.slots[0].vmax, 2);
        assert_eq!(r.participants.len(), 2);
        assert_eq!(r.participants[0].mail, "x@a");
        assert_eq!(r.participants[0].wish, vec![0, 1]);
        assert_eq!(r.participants[1].wish, vec![1, 0]);
    }

    #[test]
    fn parse_empty_input() {
        let r = parse("");
        assert!(r.errors.is_empty());
        assert!(r.slots.is_empty());
        assert!(r.participants.is_empty());
    }

    #[test]
    fn parse_comments_only() {
        let r = parse("% this is a comment\n# another comment\n");
        assert!(r.errors.is_empty());
        assert!(r.slots.is_empty());
    }

    #[test]
    fn parse_blank_lines_and_comments() {
        let text = "\n% header\n[slots]\n% a comment\n\"Slot1\" 0 5\n\n[participants]\n# another comment\n\"a@b\" 0\n";
        let r = parse(text);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
        assert_eq!(r.slots.len(), 1);
        assert_eq!(r.participants.len(), 1);
    }

    // ── parse() error cases ────────────────────────────────────────

    #[test]
    fn parse_missing_slots_section() {
        let text = "[participants]\n\"x@a\" 0\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("slots"));
    }

    #[test]
    fn parse_wrong_section_order() {
        let text = "[slots]\n\"A\" 1 2\n[slots]\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("participants"));
    }

    #[test]
    fn parse_vmin_gt_vmax() {
        let text = "[slots]\n\"A\" 5 2\n[participants]\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("max"));
    }

    #[test]
    fn parse_wrong_wish_count() {
        let text = "[slots]\n\"A\" 0 5\n\"B\" 0 5\n[participants]\n\"x@a\" 0\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("wish values"));
    }

    #[test]
    fn parse_duplicate_email() {
        let text = "[slots]\n\"A\" 0 5\n[participants]\n\"x@a\" 0\n\"x@a\" 0\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("multiple times"));
    }

    #[test]
    fn parse_too_many_participants() {
        let text = "[slots]\n\"A\" 0 1\n[participants]\n\"x@a\" 0\n\"y@b\" 0\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("Too many"));
    }

    #[test]
    fn parse_too_few_participants() {
        let text = "[slots]\n\"A\" 3 5\n[participants]\n\"x@a\" 0\n\"y@b\" 0\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("Not enough"));
    }

    #[test]
    fn parse_data_before_section() {
        let text = "\"orphan\" 1 2\n[slots]\n\"A\" 0 5\n";
        let r = parse(text);
        assert!(!r.warnings.is_empty());
        assert!(r.warnings[0].message.contains("ignored"));
    }

    // ── parse() warnings ───────────────────────────────────────────

    #[test]
    fn parse_unfair_wish_warns() {
        let text = "[slots]\n\"A\" 0 5\n\"B\" 0 5\n\"C\" 0 5\n[participants]\n\"x@a\" 2 2 0\n";
        let r = parse(text);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
        assert!(!r.warnings.is_empty());
        assert!(r.warnings[0].message.contains("not fair"));
    }

    #[test]
    fn parse_fair_wish_no_warning() {
        let text = "[slots]\n\"A\" 0 5\n\"B\" 0 5\n\"C\" 0 5\n[participants]\n\"x@a\" 0 1 2\n";
        let r = parse(text);
        assert!(r.errors.is_empty());
        assert!(r.warnings.is_empty(), "warnings: {:?}", r.warnings);
    }

    // ── parse() with trailing comments on data lines ───────────────

    #[test]
    fn parse_trailing_comments() {
        let text = "[slots]\n\"Monday\" 0 5  % first slot\n\"Tuesday\" 0 5  # second\n\n[participants]\n\"a@b\" 0 1  % mail sent\n";
        let r = parse(text);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
        assert_eq!(r.slots.len(), 2);
        assert_eq!(r.slots[0].name, "Monday");
        assert_eq!(r.participants[0].wish, vec![0, 1]);
    }

    // ── to_editor_text ─────────────────────────────────────────────

    #[test]
    fn to_editor_text_basic() {
        let slots = vec![
            Slot {
                name: "A".into(),
                vmin: 1,
                vmax: 5,
            },
            Slot {
                name: "B".into(),
                vmin: 0,
                vmax: 3,
            },
        ];
        let participants = vec![
            Participant {
                id: "abc123".into(),
                mail: "alice@x".into(),
                wish: vec![0, 1],
                event: String::new(),
                status: ParticipantStatus::Modified,
            },
            Participant {
                id: String::new(),
                mail: "bob@y".into(),
                wish: vec![1, 0],
                event: String::new(),
                status: ParticipantStatus::New,
            },
        ];
        let text = to_editor_text(&slots, &participants);

        assert!(text.contains("[slots]"));
        assert!(text.contains("[participants]"));
        assert!(text.contains("\"A\""));
        assert!(text.contains("\"B\""));
        assert!(text.contains("\"alice@x\""));
        assert!(text.contains("\"bob@y\""));
        assert!(text.contains("modified wishes"));
        assert!(text.contains("mail not sent"));
    }

    // ── round-trip: to_editor_text -> parse ────────────────────────

    #[test]
    fn round_trip_editor_parse() {
        let slots = vec![
            Slot {
                name: "Morning".into(),
                vmin: 1,
                vmax: 3,
            },
            Slot {
                name: "Afternoon".into(),
                vmin: 1,
                vmax: 3,
            },
        ];
        let participants = vec![
            Participant {
                id: "id1".into(),
                mail: "a@x".into(),
                wish: vec![0, 1],
                event: String::new(),
                status: ParticipantStatus::Mailed,
            },
            Participant {
                id: "id2".into(),
                mail: "b@y".into(),
                wish: vec![1, 0],
                event: String::new(),
                status: ParticipantStatus::Modified,
            },
        ];

        let text = to_editor_text(&slots, &participants);
        let parsed = parse(&text);

        assert!(
            parsed.errors.is_empty(),
            "round-trip errors: {:?}\ntext:\n{text}",
            parsed.errors
        );
        assert_eq!(parsed.slots.len(), 2);
        assert_eq!(parsed.slots[0].name, "Morning");
        assert_eq!(parsed.slots[0].vmin, 1);
        assert_eq!(parsed.slots[0].vmax, 3);
        assert_eq!(parsed.slots[1].name, "Afternoon");
        assert_eq!(parsed.participants.len(), 2);
        assert_eq!(parsed.participants[0].mail, "a@x");
        assert_eq!(parsed.participants[0].wish, vec![0, 1]);
        assert_eq!(parsed.participants[1].mail, "b@y");
        assert_eq!(parsed.participants[1].wish, vec![1, 0]);
    }

    // ── format_results ─────────────────────────────────────────────

    #[test]
    fn format_results_zero_score() {
        let slots = vec![
            Slot {
                name: "A".into(),
                vmin: 1,
                vmax: 2,
            },
            Slot {
                name: "B".into(),
                vmin: 1,
                vmax: 2,
            },
        ];
        let participants = vec![("alice@x".into(), vec![0, 1]), ("bob@y".into(), vec![1, 0])];
        let result = vec![0, 1]; // alice->A(w=0), bob->B(w=0) — score=0

        let text = format_results(&slots, &participants, &result);

        assert!(text.contains("[statistics]"));
        assert!(text.contains("\"total score\" 0"));
        assert!(text.contains("[results]"));
        assert!(text.contains("\"alice@x\""));
        assert!(text.contains("\"bob@y\""));
    }

    #[test]
    fn format_results_nonzero_score() {
        let slots = vec![
            Slot {
                name: "X".into(),
                vmin: 1,
                vmax: 2,
            },
            Slot {
                name: "Y".into(),
                vmin: 1,
                vmax: 2,
            },
        ];
        let participants = vec![("a@a".into(), vec![0, 2]), ("b@b".into(), vec![1, 0])];
        let result = vec![1, 0]; // a->Y(w=2), b->X(w=1) — score = 4+1=5

        let text = format_results(&slots, &participants, &result);
        assert!(text.contains("\"total score\" 5"));
    }

    // ── parse_quoted_string (internal) ─────────────────────────────

    #[test]
    fn quoted_string_basic() {
        assert_eq!(
            parse_quoted_string("\"hello\" rest"),
            Some(("hello", "rest"))
        );
    }

    #[test]
    fn quoted_string_empty_content() {
        assert_eq!(parse_quoted_string("\"\" rest"), Some(("", "rest")));
    }

    #[test]
    fn quoted_string_no_opening_quote() {
        assert_eq!(parse_quoted_string("noquote"), None);
    }

    #[test]
    fn quoted_string_unclosed() {
        assert_eq!(parse_quoted_string("\"unclosed"), None);
    }

    // ── parse → compute → format_results integration ──────────────

    #[test]
    fn parse_then_format_results_round_trip() {
        use crate::hungarian;

        let text = r#"[slots]
"Morning"   1 3
"Afternoon" 1 3

[participants]
"alice@x" 0 1
"bob@y"   1 0
"carol@z" 0 1
"#;
        let parsed = parse(text);
        assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

        let slots_data: Vec<(u32, u32)> = parsed.slots.iter().map(|s| (s.vmin, s.vmax)).collect();
        let n = parsed.participants.len();
        let wishes: Vec<Vec<i32>> = parsed.participants.iter().map(|p| p.wish.clone()).collect();

        let cost = hungarian::build_cost_matrix(&wishes, &slots_data, n);
        let assignment = hungarian::hungarian(&cost);
        let slot_indices = hungarian::assignment_to_slots(&assignment, &slots_data, n);

        // Every participant must be assigned to a valid slot
        for &si in &slot_indices {
            assert!(si < parsed.slots.len(), "invalid slot index {si}");
        }

        let participants_for_results: Vec<(String, Vec<i32>)> = parsed
            .participants
            .iter()
            .map(|p| (p.mail.clone(), p.wish.clone()))
            .collect();

        let result_text = format_results(&parsed.slots, &participants_for_results, &slot_indices);
        assert!(result_text.contains("[statistics]"));
        assert!(result_text.contains("[results]"));
        assert!(result_text.contains("\"alice@x\""));
        assert!(result_text.contains("\"bob@y\""));
        assert!(result_text.contains("\"carol@z\""));
        assert!(result_text.contains("\"Morning\"") || result_text.contains("\"Afternoon\""));
    }

    #[test]
    fn parse_offline_default_text() {
        // Exact default text from offline.rs — must parse cleanly
        let text = r#"[slots]
"Monday morning"    0 10
"Monday afternoon"  0 10
"Tuesday morning"   0 10

[participants]
"alice@example.com"   0 1 2
"bob@example.com"     2 0 1
"charlie@example.com" 1 2 0
"#;
        let parsed = parse(text);
        assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
        assert!(
            parsed.warnings.is_empty(),
            "warnings: {:?}",
            parsed.warnings
        );
        assert_eq!(parsed.slots.len(), 3);
        assert_eq!(parsed.participants.len(), 3);
        assert_eq!(parsed.slots[0].name, "Monday morning");
        assert_eq!(parsed.participants[0].mail, "alice@example.com");
        assert_eq!(parsed.participants[0].wish, vec![0, 1, 2]);
    }

    #[test]
    fn format_results_all_participants_appear() {
        let slots = vec![
            Slot {
                name: "A".into(),
                vmin: 1,
                vmax: 5,
            },
            Slot {
                name: "B".into(),
                vmin: 1,
                vmax: 5,
            },
            Slot {
                name: "C".into(),
                vmin: 1,
                vmax: 5,
            },
        ];
        let participants = vec![
            ("p1@x".into(), vec![0, 1, 2]),
            ("p2@x".into(), vec![2, 0, 1]),
            ("p3@x".into(), vec![1, 2, 0]),
        ];
        let result = vec![0, 1, 2];
        let text = format_results(&slots, &participants, &result);

        // All participants in results
        assert!(text.contains("\"p1@x\""));
        assert!(text.contains("\"p2@x\""));
        assert!(text.contains("\"p3@x\""));
        // All slot names in statistics
        assert!(text.contains("\"A\""));
        assert!(text.contains("\"B\""));
        assert!(text.contains("\"C\""));
        // Score = 0^2 + 0^2 + 0^2 = 0 (everyone got first choice)
        assert!(text.contains("\"total score\" 0"));
    }

    #[test]
    fn format_results_statistics_counts_correct() {
        let slots = vec![
            Slot {
                name: "X".into(),
                vmin: 2,
                vmax: 2,
            },
            Slot {
                name: "Y".into(),
                vmin: 1,
                vmax: 1,
            },
        ];
        let participants = vec![
            ("a@a".into(), vec![0, 2]), // assigned to X, wish=0
            ("b@b".into(), vec![1, 0]), // assigned to X, wish=1
            ("c@c".into(), vec![2, 0]), // assigned to Y, wish=0
        ];
        let result = vec![0, 0, 1]; // a→X, b→X, c→Y
        let text = format_results(&slots, &participants, &result);
        // Score = 0 + 1 + 0 = 1
        assert!(text.contains("\"total score\" 1"));
    }

    #[test]
    fn to_editor_text_alignment() {
        // Slot names of different lengths should be padded for alignment
        let slots = vec![
            Slot {
                name: "A".into(),
                vmin: 1,
                vmax: 5,
            },
            Slot {
                name: "Long Name".into(),
                vmin: 0,
                vmax: 3,
            },
        ];
        let participants = vec![
            Participant {
                id: String::new(),
                mail: "short@x".into(),
                wish: vec![0, 1],
                event: String::new(),
                status: ParticipantStatus::New,
            },
            Participant {
                id: "id1".into(),
                mail: "very-long-email@example.com".into(),
                wish: vec![1, 0],
                event: String::new(),
                status: ParticipantStatus::Done,
            },
        ];
        let text = to_editor_text(&slots, &participants);
        // Should parse back without errors
        let parsed = parse(&text);
        assert!(
            parsed.errors.is_empty(),
            "alignment broke parsing: {:?}\ntext:\n{text}",
            parsed.errors
        );
        assert_eq!(parsed.slots.len(), 2);
        assert_eq!(parsed.participants.len(), 2);
    }

    #[test]
    fn parse_negative_wish_value() {
        let text = "[slots]\n\"A\" 0 5\n[participants]\n\"x@a\" -1\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("non-negative"));
    }

    #[test]
    fn parse_non_numeric_wish() {
        let text = "[slots]\n\"A\" 0 5\n[participants]\n\"x@a\" abc\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("not a valid number"));
    }

    #[test]
    fn parse_slot_non_numeric_bounds() {
        let text = "[slots]\n\"A\" abc 5\n[participants]\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("not a valid number"));
    }

    #[test]
    fn parse_many_participants_many_slots() {
        // Stress test: 10 slots, 10 participants
        let mut text = String::from("[slots]\n");
        for i in 0..10 {
            text.push_str(&format!("\"Slot{}\" 1 10\n", i));
        }
        text.push_str("\n[participants]\n");
        for i in 0..10 {
            text.push_str(&format!("\"p{}@x\"", i));
            for j in 0..10 {
                text.push_str(&format!(" {}", (i + j) % 10));
            }
            text.push('\n');
        }
        let r = parse(&text);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
        assert_eq!(r.slots.len(), 10);
        assert_eq!(r.participants.len(), 10);
    }

    #[test]
    fn parse_unclosed_section_bracket() {
        let text = "[slots\n\"A\" 1 2\n";
        let r = parse(text);
        assert!(!r.errors.is_empty());
        assert!(r.errors[0].message.contains("Unclosed"));
    }
}
