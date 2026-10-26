use protocol::usage::{Span, Usage, Window};

pub const SESSION: i64 = 5 * 60 * 60 * 1000;

pub fn span(span: Span) -> &'static str {
    match span {
        Span::Session => "5-hour",
        Span::Week => "Weekly",
        Span::Month => "Monthly",
    }
}

pub fn resets_at(window: &Window, now: i64) -> i64 {
    let at = window.resets * 1000;
    if window.span == Span::Session {
        at.min(now + SESSION)
    } else {
        at
    }
}

pub fn until(milliseconds: i64) -> String {
    let minutes = ((milliseconds + 59_999) / 60_000).max(1);
    let (days, hours) = (minutes / 1440, minutes % 1440 / 60);
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {}m", minutes % 60)
    } else {
        format!("{minutes}m")
    }
}

pub fn left(window: &Window) -> u32 {
    crate::num::floor64(window.left.clamp(0., 100.))
}

pub fn shown(usage: &Usage, now: i64) -> Option<(Vec<Window>, Window)> {
    let open: Vec<Window> = usage
        .windows
        .iter()
        .filter(|window| window.resets * 1000 > now)
        .copied()
        .collect();
    let shown = open
        .iter()
        .find(|window| window.span == Span::Session)
        .or_else(|| open.first())
        .copied()?;
    Some((open, shown))
}

pub fn newest<'a>(
    readings: impl IntoIterator<Item = &'a Usage>,
    agent: protocol::agents::Agent,
) -> Option<&'a Usage> {
    readings
        .into_iter()
        .filter(|usage| usage.agent == agent)
        .fold(None, |best: Option<&Usage>, usage| match best {
            Some(best) if best.at >= usage.at => Some(best),
            _ => Some(usage),
        })
}

#[cfg(test)]
mod tests {
    use protocol::agents::Agent;

    use super::*;

    fn window(span: Span, left: f64, resets: i64) -> Window {
        Window { span, left, resets }
    }

    #[test]
    fn formats_the_time_until_a_reset() {
        assert_eq!(until(30_000), "1m");
        assert_eq!(until(45 * 60_000), "45m");
        assert_eq!(until((2 * 60 + 3) * 60_000), "2h 3m");
        assert_eq!(until((26 * 60 + 5) * 60_000), "1d 2h");
        assert_eq!(until(-5), "1m");
    }

    #[test]
    fn a_session_resets_within_five_hours() {
        let now = 1_000_000;
        let late = window(Span::Session, 50., (now + 10 * SESSION) / 1000);
        assert_eq!(resets_at(&late, now), now + SESSION);
        let week = window(Span::Week, 50., (now + 10 * SESSION) / 1000);
        assert_eq!(resets_at(&week, now), now + 10 * SESSION);
    }

    #[test]
    fn prefers_the_session_window_and_drops_expired_ones() {
        let usage = Usage {
            agent: Agent::Claude,
            windows: vec![
                window(Span::Week, 80., 5000),
                window(Span::Session, 62.7, 4000),
                window(Span::Month, 10., 1),
            ],
            at: 1,
        };
        let (open, shown) = shown(&usage, 2_000_000).unwrap();
        assert_eq!(open.len(), 2);
        assert_eq!(shown.span, Span::Session);
        assert_eq!(left(&shown), 62);
    }

    #[test]
    fn the_newest_reading_wins_across_machines() {
        let old = Usage {
            agent: Agent::Codex,
            windows: Vec::new(),
            at: 1,
        };
        let new = Usage {
            agent: Agent::Codex,
            windows: Vec::new(),
            at: 9,
        };
        let other = Usage {
            agent: Agent::Claude,
            windows: Vec::new(),
            at: 99,
        };
        assert_eq!(
            newest([&old, &new, &other], Agent::Codex).map(|usage| usage.at),
            Some(9)
        );
    }
}
