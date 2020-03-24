use bitvec::prelude::*;

const BUFFER_SIZE: usize = 64;

fn to_index(sequence_number: u64) -> usize {
    (sequence_number % BUFFER_SIZE as u64) as usize
}

fn in_range(sequence_number: u64, next: u64) -> bool {
    if sequence_number < BUFFER_SIZE as u64 {
        true
    } else if sequence_number >= next {
        true
    } else {
        sequence_number > next - BUFFER_SIZE as u64
    }
}

#[derive(Debug)]
pub struct ReplayBuffer {
    next: u64,
    acks: BitVec,
}

impl ReplayBuffer {
    pub fn new() -> Self {
        Self {
            next: 0,
            acks: BitVec::repeat(false, BUFFER_SIZE),
        }
    }

    pub fn acknowledge(&mut self, sequence_number: u64) -> bool {
        let index = to_index(sequence_number);
        
        if sequence_number == self.next {
            self.next = sequence_number + 1;
            self.acks.set(index, true);
            true
        } else if sequence_number > self.next {
            self.next = sequence_number + 1;

            for i in self.next..sequence_number {
                let index = to_index(i);
                self.acks.set(index, false);
            }

            self.acks.set(index, true);

            true
        } else if in_range(sequence_number, self.next) {
            if self.acks[index] {
                false
            } else {
                self.acks.set(index, true);
                true
            }
        } else {
            false
        }
    }

    pub fn is_acknowledged(&self, sequence_number: u64) -> bool {
        if !in_range(sequence_number, self.next) {
            false
        } else if sequence_number >= self.next {
            false
        } else {
            let index = to_index(sequence_number);
            self.acks[index]
        }
    }

    pub fn get_ack_bits(&self) -> (u64, [u8; 4]) {
        if self.next == 0 {
            return (0, [0; 4]);
        }
        else if self.next <= 32 {
            let mut buffer: [u8; 4] = [0; 4];
            let bits = BitSlice::<Msb0, u8>::from_slice_mut(&mut buffer[..]);


            for i in 0..self.next as usize {
                bits.set(i, self.acks[i]);
            }

            return (self.next, buffer);
        } else {
            let mut buffer: [u8; 4] = [0; 4];
            let bits = BitSlice::<Msb0, u8>::from_slice_mut(&mut buffer[..]);

            for i in 0..32 {

                let seq = self.next - (32 - i as u64);
                let index = to_index(seq);

                let is_ack = self.acks[index];

                bits.set(i, is_ack);

            }

            return (self.next, buffer);
        }
    }

    pub fn set_ack_bits(&mut self, next: u64, buffer: [u8; 4]) -> Vec<u64> {
        let mut acked = Vec::new();
        let bits = BitSlice::<Msb0, u8>::from_slice(&buffer[..]);

        if next == 0 {
            // do nothing
        } else if next <= 32 {
            for i in 0..next as usize {
                if bits[i] {
                    if self.acknowledge(i as u64) {
                        acked.push(i as u64);
                    }
                }
            }

        } else {
            for i in 0..32 {
                if bits[i] {
                    let seq = next - (32 - i as u64);
                    if self.acknowledge(seq) {
                        acked.push(seq);
                    }
                }
            }
        }

        acked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {

        let mut buffer = ReplayBuffer::new();

        assert_eq!(false, buffer.is_acknowledged(0));
        assert_eq!(true, buffer.acknowledge(0));
        assert_eq!(true, buffer.is_acknowledged(0));
    }

    #[test]
    fn test_2() {
        let mut buffer = ReplayBuffer::new();

        assert_eq!(true, buffer.acknowledge(2));
        assert_eq!(false, buffer.is_acknowledged(0));
        assert_eq!(false, buffer.is_acknowledged(1));
        assert_eq!(true, buffer.is_acknowledged(2));
    }

    #[test]
    fn it_acknowledges_a_sequence_number_only_once() {
        let mut buffer = ReplayBuffer::new();

        assert_eq!(true, buffer.acknowledge(15));
        assert_eq!(false, buffer.acknowledge(15));

        assert_eq!(true, buffer.is_acknowledged(15));
    }

    #[test]
    fn it_forgets_things_outside_the_buffer() {
        let mut buffer = ReplayBuffer::new();

        buffer.acknowledge(1000);
        buffer.acknowledge(2000);

        assert_eq!(false, buffer.is_acknowledged(1000));
    }

    #[test]
    fn ack_bits_for_an_empty_buffer_are_empty() {
        let buffer = ReplayBuffer::new();

        assert_eq!((0, [0; 4]), buffer.get_ack_bits());
    }

    #[test]
    fn test_7() {
        let mut buffer = ReplayBuffer::new();

        for i in 0..30 {
            buffer.acknowledge(i);
        }

        assert_eq!((30, [255, 255, 255, 252]), buffer.get_ack_bits());
    }

    #[test]
    fn test_8() {
        let mut buffer = ReplayBuffer::new();

        for i in 60..64 {
            buffer.acknowledge(i);
        }

        assert_eq!((64, [0, 0, 0, 15]), buffer.get_ack_bits());
    }

    #[test]
    fn test_9() {
        let mut source = ReplayBuffer::new();
        let mut target = ReplayBuffer::new();

        for i in 60..64 {
            source.acknowledge(i);
        }

        let (next, buffer) = source.get_ack_bits();

        target.set_ack_bits(next, buffer);

        assert_eq!(false, target.is_acknowledged(59));
        assert_eq!(true, target.is_acknowledged(60));
        assert_eq!(true, target.is_acknowledged(61));
        assert_eq!(true, target.is_acknowledged(62));
        assert_eq!(true, target.is_acknowledged(63));
        assert_eq!(false, target.is_acknowledged(64));
    }

    #[test]
    fn test_10() {
        let mut source = ReplayBuffer::new();
        let mut target = ReplayBuffer::new();

        for i in 16..30 {
            source.acknowledge(i);
        }

        let (next, buffer) = source.get_ack_bits();

        target.set_ack_bits(next, buffer);

        assert_eq!(false, target.is_acknowledged(15));
        assert_eq!(true, target.is_acknowledged(16));
        assert_eq!(true, target.is_acknowledged(17));
        assert_eq!(true, target.is_acknowledged(18));
        assert_eq!(true, target.is_acknowledged(19));
        assert_eq!(true, target.is_acknowledged(20));
        assert_eq!(true, target.is_acknowledged(21));
        assert_eq!(true, target.is_acknowledged(22));
        assert_eq!(true, target.is_acknowledged(23));
        assert_eq!(true, target.is_acknowledged(24));
        assert_eq!(true, target.is_acknowledged(25));
        assert_eq!(true, target.is_acknowledged(26));
        assert_eq!(true, target.is_acknowledged(27));
        assert_eq!(true, target.is_acknowledged(28));
        assert_eq!(true, target.is_acknowledged(29));
        assert_eq!(false, target.is_acknowledged(30));
    }
}
