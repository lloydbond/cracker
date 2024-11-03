use peg::*;

parser!(pub grammar makefile() for str {
   #[no_eof]
   pub rule Targets() -> Vec<String> = a:Target()++ " " ":"!"=" { a }
       rule Target() -> String = Spacing() a:['a'..='z'|'A'..='Z'] t:['a'..='z'|'A'..='Z'|'_']+ {
            let res: String = [a].iter().chain(&t).collect();
           res
       }
       rule Spacing() = quiet!{[' ']*}
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_phony() {
        let rule = ".PHONY: target1 target2";
        let targets = makefile::Targets(rule).ok();
        assert_eq!(targets, None);
    }
}
