use std::{path::{PathBuf}, collections::VecDeque}; 
use tui::{
    widgets::{ListState},
};

pub struct Queue {
    state: ListState,
    items: VecDeque<PathBuf>,
    curr: usize,
}

impl Queue {

    pub fn with_items() -> Queue {
        Queue {
            state: ListState::default(),
            items: VecDeque::new(),
            curr: 0,
        }
    }

    // return all items contained in vector
    pub fn get_items(&self) -> &VecDeque<PathBuf> {
        &self.items
    }

    // return item at index
    pub fn get_item(&self) -> &PathBuf {
        &self.items[self.curr]
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    // FIFO
    pub fn pop(&mut self) -> Option<PathBuf>{
        self.items.pop_front()
    }

    pub fn length(&self) -> usize {
        self.items.len()
    }

    pub fn get_state(&self) -> ListState {
        self.state.clone()
    }

    pub fn next(&mut self) { 
        // check if empty
        if self.items.is_empty(){return};

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
        // check if empty
        if self.items.is_empty(){return};

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


    pub fn add(&mut self, item: PathBuf){

        if item.is_dir(){
            return;
        } else {
            self.items.push_back(item);
        }
    }

    // remove item from items vector
    pub fn remove(&mut self){

        // if list is empty ignore
        if self.items.len() == 0{
            return;
        // top of queue
        } else if self.items.len() == 1 {
            self.items.remove(self.curr);
            self.unselect();
        // if at bottom of queue, remove item and select item above above
        } else if (self.state.selected().unwrap()) >= (self.items.len() - 1){
            self.items.remove(self.curr);
            self.curr -= 1;
            self.state.select(Some(self.curr));
        // else delete item
        } else if !(self.items.is_empty()){
            self.items.remove(self.curr);
        };
    }

}
