use std::path::PathBuf;

pub struct NavigationState {
    pub current: PathBuf,
    back_stack: Vec<PathBuf>,
    forward_stack: Vec<PathBuf>,
}

impl NavigationState {
    pub fn new(start: PathBuf) -> Self {
        Self {
            current: start,
            back_stack: Vec::new(),
            forward_stack: Vec::new(),
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        self.back_stack.push(self.current.clone());
        self.current = path;
        self.forward_stack.clear();
    }

    pub fn go_back(&mut self) -> bool {
        if let Some(prev) = self.back_stack.pop() {
            self.forward_stack.push(self.current.clone());
            self.current = prev;
            true
        } else {
            false
        }
    }

    pub fn go_forward(&mut self) -> bool {
        if let Some(next) = self.forward_stack.pop() {
            self.back_stack.push(self.current.clone());
            self.current = next;
            true
        } else {
            false
        }
    }

    pub fn go_up(&mut self) -> bool {
        if let Some(parent) = self.current.parent() {
            let parent = parent.to_path_buf();
            if parent != self.current {
                self.navigate_to(parent);
                return true;
            }
        }
        false
    }

    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }
}
