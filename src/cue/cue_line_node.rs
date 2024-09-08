use std::collections::VecDeque;

use crate::cue::cue_line::CueLine;

#[derive(Debug)]
pub struct CueLineNode {
    pub line: Option<CueLine>,
    pub children: Vec<CueLineNode>,
}

impl CueLineNode {
    fn from_line(line: CueLine) -> Self {
        Self {
            line: Some(line),
            children: Vec::new(),
        }
    }

    pub fn from_lines(mut cue_lines: VecDeque<CueLine>) -> Vec<CueLineNode> {
        let mut stack = Vec::new();

        stack.push(CueLineNode {
            line: None,
            children: Vec::new(),
        });

        while let Some(line) = cue_lines.pop_front() {
            let Some(stack_top) = stack.last_mut() else {
                panic!("Stack should always have at least one element!")
            };

            let stack_top_indentation = match &stack_top.line {
                Some(parent_line) => parent_line.indentation + 2,
                None => 0,
            };
            let line_indentation = line.indentation + 2;

            if line_indentation >= stack_top_indentation {
                stack.push(CueLineNode::from_line(line))
            } else {
                let mut v = Vec::new();

                while let Some(node) = stack.pop() {
                    if node.line.as_ref().is_none()
                        || node
                            .line
                            .as_ref()
                            .is_some_and(|l| l.indentation + 2 != stack_top_indentation)
                    {
                        stack.push(node);
                        break;
                    } else {
                        v.push(node);
                    }
                }

                v.reverse();
                stack.last_mut().unwrap().children = v;
                stack.push(CueLineNode::from_line(line));
            }
        }

        while stack.len() > 1 {
            let mut tmp_children = Vec::new();
            let mut depth = 0;

            while let Some(mut node) = stack.pop() {
                if node.line.is_none() {
                    tmp_children.reverse();
                    node.children.append(&mut tmp_children);
                    stack.push(node);
                    break;
                } else {
                    if tmp_children.is_empty() {
                        depth = node.line.as_ref().unwrap().indentation;
                        tmp_children.push(node);
                    } else if node.line.as_ref().unwrap().indentation == depth {
                        tmp_children.push(node);
                    } else {
                        tmp_children.reverse();
                        node.children.append(&mut tmp_children);
                        stack.push(node);
                    }
                }
            }
        }

        stack.pop().unwrap().children
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cue_line_nodes_from_lines() {
        let path = Path::new("src/cue/Tim Buckley - Happy Sad.cue");
        let cue_lines = CueLine::from_file(&path).unwrap();

        let cue_nodes = CueLineNode::from_lines(VecDeque::from(cue_lines));

        assert_eq!(cue_nodes.len(), 7);

        let keys: Vec<String> = cue_nodes.iter().map(|n| n.line.as_ref().unwrap().key.clone()).collect();
        assert_eq!(keys, vec!["REM", "REM", "REM", "REM", "PERFORMER", "TITLE", "FILE"]);

        let file = cue_nodes.last().unwrap();
        assert_eq!(file.line.as_ref().unwrap().key, "FILE");
        assert_eq!(file.children.len(), 6);

        assert!(file.children.iter().all(|e| e.line.as_ref().unwrap().key == "TRACK"));

        let track = file.children[0].line.as_ref().unwrap();

        assert_eq!(track.key, "TRACK");
        assert_eq!(track.value, "01 AUDIO");
        assert_eq!(file.children[0].children.len(), 3);

        assert!(file.children[0].children[0].children.is_empty());
        assert_eq!(
            file.children[0].children[0].line,
            Some(CueLine {
                indentation: 4,
                key: "TITLE".to_string(),
                value: "\"Strange Feelin'\"".to_string(),
            })
        );
        assert_eq!(
            file.children[0].children[1].line,
            Some(CueLine {
                indentation: 4,
                key: "PERFORMER".to_string(),
                value: "\"Tim Buckley\"".to_string(),
            })
        );
        assert_eq!(
            file.children[0].children[2].line,
            Some(CueLine {
                indentation: 4,
                key: "INDEX".to_string(),
                value: "01 00:00:00".to_string(),
            })
        );
    }
}
