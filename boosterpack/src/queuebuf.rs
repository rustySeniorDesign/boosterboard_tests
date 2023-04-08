//! Circular buffer datastructure implementation.


use core::panicking::panic;


// Size of buf needs to be a power of two to avoid calculating a modulo when incrementing
pub struct QueueBuf<const Size: usize>{
    buf:  [u8;Size],
    mask: u16,
    curr: u16, //ptr to current slot to get from
    next: u16, //ptr to next available slot
    used: u16,
}

impl<const Size: usize> QueueBuf<Size>{
    pub const fn new(arr: [u8;Size]) -> Self{
        let len = arr.len();
        QueueBuf{
            buf: arr,
            mask: (len as u16) - 1u16,
            curr: 0,
            next: 0,
            used: 0,
        }
    }

    #[inline]
    fn inc(&self, val:u16) -> u16{
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
    pub fn slots_used(&self) -> u16{
        self.used
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
        self.used += 1;
    }

    //make sure to check for data before calling
    pub fn get(&mut self) -> u8{
        let val = self.buf[self.curr as usize];
        self.curr = self.inc(self.curr);
        self.used -= 1;
        val
    }
}