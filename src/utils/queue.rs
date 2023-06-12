 pub struct Queue<T>{
     items: Vec<T>,
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Queue<T> {
    pub fn new()-> Self{
         Queue{items: Vec::new()}
     }

     pub fn enqueue(&mut self, item:T){
         self.items.push(item);
     }

     pub fn dequeue(&mut self) -> Option<T>{
         Some(self.items.remove(0))
     }

     pub fn is_empty(&self) -> bool{
         self.items.is_empty()
     }

     pub fn size(&self) -> usize{
         self.items.len()
     }
}
