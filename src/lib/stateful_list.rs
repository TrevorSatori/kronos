use std::{fs, path::{PathBuf, Path}}; 
extern crate glob;
use glob::{glob, glob_with, MatchOptions, Pattern};
use std::env;

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};
use std::fs::File;
use std::io::BufReader;
use rodio::{Sink, Decoder, OutputStream, source::Source};
use std::ffi::OsStr;

use crate::*;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
    pub curr: usize,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
            curr: 0,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.curr = i;
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.curr = i;
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
