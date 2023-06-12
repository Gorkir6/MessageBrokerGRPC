 pub struct Queue<T>{
     items: Vec<T>,
}

impl<T> Queue<T> {
    pub fn new()-> Self{
         Queue{items: Vec::new()}
     }

     pub fn enqueue(&mut self, item:T){
         self.items.push(item);
     }

     pub fn dequeue(&mut self) -> Option<T>{
         self.items.remove(0)
     }

     pub fn is_empty(&self) -> bool{
         self.items.is_empty()
     }

     pub fn size(&self) -> usize{
         self.items.len()
     }
}
