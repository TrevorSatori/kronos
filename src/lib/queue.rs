use std::{path::{PathBuf, Path}, collections::VecDeque, time::Duration}; 
use lofty::{Probe, AudioFile};
use tui::{
    widgets::{ListState},
};

use super::gen_funcs::{bulk_add};
pub struct Queue {
    state: ListState,
    items: VecDeque<(PathBuf, u16)>,
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

     // return item at index
    pub fn get_item(&self) -> &(PathBuf, u16) {
        &self.items[self.curr]
    }

    // return all items contained in vector
    pub fn get_items(&self) -> &VecDeque<(PathBuf, u16)> {
        &self.items
    }

    pub fn get_length(&self) -> usize {
        self.items.len()
    }

    pub fn get_total_time(&self)  -> String {

        let mut total_time = 0;


        for i in &self.items {
            total_time += i.1 as u64; 
        }

        // days
        if total_time / 86400 >= 1 {
    
        let days = total_time / 86400;
        let hours = (total_time % 86400) / 3600;
        let minutes = (total_time %  3600) / 60;
        
        return days.to_string() + " days " + &hours.to_string() + " hours " + &minutes.to_string() + " minutes |"

        // hours
        } else if total_time / 3600 >= 1 {

            let hours = total_time / 3600;
            let minutes = (total_time %  3600) / 60;
            let seconds = total_time % 60;

            return hours.to_string() + " hours " + &minutes.to_string() + " minutes " + &seconds.to_string() + " seconds |";  

        // minutes
        } else if total_time / 60 >= 1 {

            let minutes = total_time / 60;
            let seconds = total_time % 60;

            return minutes.to_string() + " minutes " + &seconds.to_string() + " seconds |";  
        // seconds
        } else {
            return total_time.to_string() + " seconds |";
        } 
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn pop(&mut self) -> Option<(PathBuf, u16)>{
        self.items.pop_front()
    }


    pub fn get_state(&self) -> ListState {
        self.state.clone()
    }


    // get audio file length
    pub fn item_length(&mut self, path: &PathBuf) -> u16{

        let path = Path::new(&path);
        let tagged_file = Probe::open(path)
		.expect("ERROR: Bad path provided!")
		.read()
		.expect("ERROR: Failed to read file!");

        let properties = &tagged_file.properties();
	    let duration = properties.duration();
        
        // update song length, currently playing
        duration.as_secs() as u16
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
            let files = bulk_add(&item);
            for f in files{
                let length = self.item_length(&f);
                self.items.push_back((f, length));    
            }
        } else {
            let length = self.item_length(&item);
            self.items.push_back((item, length));
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
