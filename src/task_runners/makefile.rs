pub mod parser {
    use peg::*;

    parser!(pub grammar grammar() for str {
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
            let actual: Vec<String> = grammar::Targets(rule).ok().unwrap();
            let expected: Vec<String> = vec![String::from("target")];
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_multi_target_rule() {
            let rule = "target t target_three:";
            let actual: Vec<String> = grammar::Targets(rule).ok().unwrap();
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
            let actual: Vec<String> = grammar::Targets(rule).ok().unwrap();
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
            let actual: Vec<String> = grammar::Targets(rule).ok().unwrap();
            let expected: Vec<String> = vec![String::from("1target"), String::from("target1")];
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_target_with_dot() {
            let rule = "target.o:";
            let actual: Vec<String> = grammar::Targets(rule).ok().unwrap();
            let expected: Vec<String> = vec![String::from("target.o")];
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_target_inline_recipe_rule() {
            let rule = "target: ; tail -f /dev/null";
            let actual: Vec<String> = grammar::Targets(rule).ok().unwrap();
            let expected: Vec<String> = vec![String::from("target")];
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_ignore_phony() {
            let rule = ".PHONY: target1 target2";
            let actual = grammar::Targets(rule).ok();
            let expected = None;
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_ignore_guard() {
            let rule = "_guard: target1 target2";
            let actual = grammar::Targets(rule).ok();
            let expected = None;
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_ignore_variable() {
            let rule = "files := file1 file2";
            let actual = grammar::Targets(rule).ok();
            let expected = None;
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_ignore_wildcard_catchall() {
            let rule = "%::";
            let actual = grammar::Targets(rule).ok();
            let expected = None;
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_ignore_tabbed_target() {
            let rule = "\ttarget:";
            let actual = grammar::Targets(rule).ok();
            let expected = None;
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_recipe_should_err() {
            let rule = "\t@echo \"Hello Cracker\"";
            assert!(grammar::Targets(rule).is_err());
        }
    }
}

pub mod worker {
    use iced::futures::{SinkExt, Stream, StreamExt};
    use iced::stream::try_channel;
    use iced::Subscription;

    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    use std::hash::Hash;
    use std::process::Stdio;
    use std::sync::Arc;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Stdout {
        Prepare { output: String },
        OutputUpdate { output: String },
        Finished,
    }

    #[derive(Debug, Clone)]
    pub enum Error {
        Failed(Arc<std::io::Error>),
        NoContent,
    }

    impl From<std::io::Error> for Error {
        fn from(error: std::io::Error) -> Self {
            Error::Failed(Arc::new(error))
        }
    }

    pub fn subscription<I: 'static + Hash + Copy + Send + Sync>(
        id: I,
        target: String,
    ) -> Subscription<(I, Result<Stdout, Error>)> {
        Subscription::run_with_id(
            id,
            some_worker(target.clone()).map(move |output| (id, output)),
        )
    }

    pub fn some_worker(target: String) -> impl Stream<Item = Result<Stdout, Error>> {
        try_channel(1, |mut output| async move {
            let _ = output
                .send(Stdout::OutputUpdate {
                    output: "".to_string(),
                })
                .await;
            debug!("initialize worker: {target}");
            let mut cmd = Command::new("make");

            // Specify that we want the command's standard output piped back to us.
            // By default, standard input/output/error will be inherited from the
            // current process (for example, this means that standard input will
            // come from the keyboard and standard output/error will go directly to
            // the terminal if this process is invoked from the command line).
            cmd.stdout(Stdio::piped());
            cmd.stdin(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = cmd
                .arg(target.as_str())
                .spawn()
                .expect("failed to spawn command");

            let stdout = child
                .stdout
                .take()
                .expect("child did not have a handle to stdout");
            let mut reader = BufReader::new(stdout).lines();

            while let result = reader.next_line().await {
                use iced::futures::StreamExt;
                match result {
                    Ok(line) => match line {
                        Some(l) => {
                            let _ = output.send(Stdout::OutputUpdate { output: l }).await;
                        }
                        None => {
                            debug!("data stream ended");
                            break;
                        }
                    },
                    Err(_) => {
                        error!("file stream error:");
                    }
                }
            }
            let _ = output.send(Stdout::Finished).await;
            debug!("leaving worker");
            Ok(())
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        // TODO: make this testable if possible
    }
}
