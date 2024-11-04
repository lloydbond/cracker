use peg::*;

parser!(pub grammar makefile() for str {
   #[no_eof]
   pub rule Targets() -> Vec<String> = a:Target()++ " " ":"!"=" { a }
       rule Target() -> String = Spacing() a:['a'..='z'|'A'..='Z'|'0'..='9'] t:['a'..='z'|'A'..='Z'|'0'..='9'|'_'|'.']* {
            let res: String = [a].iter().chain(&t).collect();
           res
       }
       rule Spacing() = quiet!{[' ']*}
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_target_rule() {
        let rule = "target:";
        let actual: Vec<String> = makefile::Targets(rule).ok().unwrap();
        let expected: Vec<String> = vec![String::from("target")];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_multi_target_rule() {
        let rule = "target t target_three:";
        let actual: Vec<String> = makefile::Targets(rule).ok().unwrap();
        let expected: Vec<String> = vec![
            String::from("target"),
            String::from("t"),
            String::from("target_three"),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_multi_target_variable_spacing_rule() {
        let rule = " target  t       target_three:";
        let actual: Vec<String> = makefile::Targets(rule).ok().unwrap();
        let expected: Vec<String> = vec![
            String::from("target"),
            String::from("t"),
            String::from("target_three"),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_targets_with_num() {
        let rule = "1target target1:";
        let actual: Vec<String> = makefile::Targets(rule).ok().unwrap();
        let expected: Vec<String> = vec![String::from("1target"), String::from("target1")];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_target_with_dot() {
        let rule = "target.o:";
        let actual: Vec<String> = makefile::Targets(rule).ok().unwrap();
        let expected: Vec<String> = vec![String::from("target.o")];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_target_inline_recipe_rule() {
        let rule = "target: ; tail -f /dev/null";
        let actual: Vec<String> = makefile::Targets(rule).ok().unwrap();
        let expected: Vec<String> = vec![String::from("target")];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_ignore_phony() {
        let rule = ".PHONY: target1 target2";
        let actual = makefile::Targets(rule).ok();
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_ignore_guard() {
        let rule = "_guard: target1 target2";
        let actual = makefile::Targets(rule).ok();
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_ignore_variable() {
        let rule = "files := file1 file2";
        let actual = makefile::Targets(rule).ok();
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_ignore_wildcard_catchall() {
        let rule = "%::";
        let actual = makefile::Targets(rule).ok();
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_ignore_tabbed_target() {
        let rule = "\ttarget:";
        let actual = makefile::Targets(rule).ok();
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_recipe_should_err() {
        let rule = "\t@echo \"Hello Cracker\"";
        assert!(makefile::Targets(rule).is_err());
    }
}
