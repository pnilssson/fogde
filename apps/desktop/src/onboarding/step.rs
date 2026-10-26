#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Where,
    Server,
    Github,
    Agents,
    Repos,
    Clone,
}

const ORDER: [Step; 6] = [
    Step::Where,
    Step::Server,
    Step::Github,
    Step::Agents,
    Step::Repos,
    Step::Clone,
];

pub fn steps(server: bool) -> Vec<Step> {
    ORDER
        .into_iter()
        .filter(|step| server || *step != Step::Server)
        .collect()
}

pub fn first(onboarded: bool, machines: &[String]) -> (Step, Option<String>) {
    match machines {
        [only] if !onboarded => (Step::Github, Some(only.clone())),
        _ => (Step::Where, None),
    }
}

pub fn forward(from: Step, to: Step) -> bool {
    let index = |step| ORDER.iter().position(|known| *known == step);
    index(to) > index(from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_computer_skips_the_server_step() {
        assert_eq!(
            steps(false),
            [
                Step::Where,
                Step::Github,
                Step::Agents,
                Step::Repos,
                Step::Clone
            ]
        );
        assert_eq!(steps(true).len(), 6);
    }

    #[test]
    fn a_single_machine_resumes_at_github_until_onboarded() {
        let one = ["local".to_owned()];
        assert_eq!(first(false, &one), (Step::Github, Some("local".into())));
        assert_eq!(first(true, &one), (Step::Where, None));
        assert_eq!(first(false, &[]), (Step::Where, None));
    }

    #[test]
    fn steps_know_their_direction() {
        assert!(forward(Step::Where, Step::Github));
        assert!(!forward(Step::Clone, Step::Repos));
    }
}
