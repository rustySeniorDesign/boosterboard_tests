//! Circular buffer datastructure implementation.


pub struct QueueBuf{
    // Size of buf needs to be a power of two to avoid calculating a modulo when incrementing
    buf:  &'static mut [u8],
    mask: u16,
    curr: u16, //ptr to current slot to get from
    next: u16, //ptr to next available slot
}

impl QueueBuf{
    pub fn new(arr: &'static mut [u8]) -> Self{
        QueueBuf{
            buf: arr,
            mask: (arr.len() as u16) - 1u8,
            curr: 0,
            next: 0,
        }
    }

    #[inline]
    fn inc(self, val:u16) -> u16{
        (val+1u16) & self.mask
    }

    #[inline]
    pub fn has_data(&self) -> bool{
        return self.curr != self.next;
    }

    #[inline]
    pub fn slots_left(&self) -> u16{
        self.mask - (self.next - self.curr)
    }

    #[inline]
    pub fn is_full(&self) -> bool{
        return self.curr == self.inc(self.next);
    }

    #[inline]
    pub fn is_empty(&self) -> bool{
        return  self.curr == self.next;
    }

    //make sure to check for fullness before calling
    pub fn put(&mut self, val: u8){
        self.buf[self.next as usize] = val;
        self.next = self.inc(self.next);
    }

    //make sure to check for data before calling
    pub fn get(&mut self) -> u8{
        let val = self.buf[self.curr as usize];
        self.curr = self.inc(self.curr);
        val
    }
}