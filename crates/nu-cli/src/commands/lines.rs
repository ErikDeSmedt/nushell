use crate::commands::WholeStreamCommand;
use crate::prelude::*;
use nu_errors::ShellError;
use nu_protocol::{Primitive, ReturnSuccess, Signature, UntaggedValue, Value};

pub struct Lines;

#[async_trait]
impl WholeStreamCommand for Lines {
    fn name(&self) -> &str {
        "lines"
    }

    fn signature(&self) -> Signature {
        Signature::build("lines")
    }

    fn usage(&self) -> &str {
        "Split single string into rows, one per line."
    }

    async fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        lines(args, registry).await
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Split multi-line string into lines",
            example: r#"^echo "two\nlines" | lines"#,
            result: None,
        }]
    }
}

fn ends_with_line_ending(st: &str) -> bool {
    let mut temp = st.to_string();
    let last = temp.pop();
    if let Some(c) = last {
        c == '\n'
    } else {
        false
    }
}

async fn lines(args: CommandArgs, registry: &CommandRegistry) -> Result<OutputStream, ShellError> {
    let leftover = Arc::new(vec![]);
    let leftover_string = Arc::new(String::new());
    let registry = registry.clone();
    let args = args.evaluate_once(&registry).await?;
    let tag = args.name_tag();
    let name_span = tag.span;

    let eos = futures::stream::iter(vec![
        UntaggedValue::Primitive(Primitive::EndOfStream).into_untagged_value()
    ]);

    Ok(args
        .input
        .chain(eos)
        .map(move |item| {
            let mut leftover = leftover.clone();
            let mut leftover_string = leftover_string.clone();

            match item {
                Value {
                    value: UntaggedValue::Primitive(Primitive::String(st)),
                    ..
                } => {
                    let st = (&*leftover_string).clone() + &st;
                    if let Some(leftover) = Arc::get_mut(&mut leftover) {
                        leftover.clear();
                    }

                    let mut lines: Vec<String> = st.lines().map(|x| x.to_string()).collect();

                    if !ends_with_line_ending(&st) {
                        if let Some(last) = lines.pop() {
                            if let Some(leftover_string) = Arc::get_mut(&mut leftover_string) {
                                leftover_string.clear();
                                leftover_string.push_str(&last);
                            }
                        } else if let Some(leftover_string) = Arc::get_mut(&mut leftover_string) {
                            leftover_string.clear();
                        }
                    } else if let Some(leftover_string) = Arc::get_mut(&mut leftover_string) {
                        leftover_string.clear();
                    }

                    let success_lines: Vec<_> = lines
                        .iter()
                        .map(|x| ReturnSuccess::value(UntaggedValue::line(x).into_untagged_value()))
                        .collect();

                    futures::stream::iter(success_lines)
                }
                Value {
                    value: UntaggedValue::Primitive(Primitive::Line(st)),
                    ..
                } => {
                    let st = (&*leftover_string).clone() + &st;
                    if let Some(leftover) = Arc::get_mut(&mut leftover) {
                        leftover.clear();
                    }
                    let mut lines: Vec<String> = st.lines().map(|x| x.to_string()).collect();
                    if !ends_with_line_ending(&st) {
                        if let Some(last) = lines.pop() {
                            if let Some(leftover_string) = Arc::get_mut(&mut leftover_string) {
                                leftover_string.clear();
                                leftover_string.push_str(&last);
                            }
                        } else if let Some(leftover_string) = Arc::get_mut(&mut leftover_string) {
                            leftover_string.clear();
                        }
                    } else if let Some(leftover_string) = Arc::get_mut(&mut leftover_string) {
                        leftover_string.clear();
                    }

                    let success_lines: Vec<_> = lines
                        .iter()
                        .map(|x| ReturnSuccess::value(UntaggedValue::line(x).into_untagged_value()))
                        .collect();
                    futures::stream::iter(success_lines)
                }
                Value {
                    value: UntaggedValue::Primitive(Primitive::EndOfStream),
                    ..
                } => {
                    if !leftover.is_empty() {
                        let mut st = (&*leftover_string).clone();
                        if let Ok(extra) = String::from_utf8((&*leftover).clone()) {
                            st.push_str(&extra);
                        }
                        // futures::stream::iter(vec![ReturnSuccess::value(
                        //     UntaggedValue::string(st).into_untagged_value(),
                        // )])
                        if !st.is_empty() {
                            futures::stream::iter(vec![ReturnSuccess::value(
                                UntaggedValue::string(&*leftover_string).into_untagged_value(),
                            )])
                        } else {
                            futures::stream::iter(vec![])
                        }
                    } else {
                        futures::stream::iter(vec![])
                    }
                }
                Value {
                    tag: value_span, ..
                } => futures::stream::iter(vec![Err(ShellError::labeled_error_with_secondary(
                    "Expected a string from pipeline",
                    "requires string input",
                    name_span,
                    "value originates from here",
                    value_span,
                ))]),
            }
        })
        .flatten()
        .to_output_stream())
}

#[cfg(test)]
mod tests {
    use super::Lines;

    #[test]
    fn examples_work_as_expected() {
        use crate::examples::test as test_examples;

        test_examples(Lines {})
    }
}
