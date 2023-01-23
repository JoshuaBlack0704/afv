use core::mem::size_of;
use core::slice::from_raw_parts;

/// Breaks down a u8 to its set of 8 bits
#[derive(Clone, Copy)]
pub struct Bits{
    src: u8,
    bits_numeric: [u8;8],
    bits_boolean: [bool;8],
}

impl Bits{
    /// Given a byte (u8) creates a Bits structure
    pub fn new(src: &u8) -> Bits {
        let src = *src;
        let mut bits_boolean = [false;8];
        let mut bits_numeric = [0u8;8];
        for bit in 0..u8::BITS as usize{
            let mask = 0x80 >> bit;
            bits_boolean[bit] = src & mask == mask;
            if src & mask == mask{
                bits_numeric[bit] = 1;
            }
            else{
                bits_numeric[bit] = 0;
            }
        }
        
        Bits{
            src,
            bits_numeric,
            bits_boolean,
        }
    }
    /// Gets the bits as a list of numbers
    pub fn bits_numeric(&self) -> &[u8; 8] {
        &self.bits_numeric
    }
    /// Gets the associated byte
    pub fn byte(&self) -> u8 {
        self.src
    }
    /// Gets the bits as a list of bools
    pub fn bits_boolean(&self) -> &[bool; 8] {
        &self.bits_boolean
    }
    /// Returns if a particular bit is set
    pub fn is_set(&self, bit_index: usize) -> bool {
        self.bits_boolean[bit_index]
    }
    /// Gets iterator over numeric bits
    pub fn numeric_iter(&self) -> core::slice::Iter<u8> {
        self.bits_numeric.iter()
    }
    /// Gets iterator over boolean bits
    pub fn boolean_iter(&self) -> core::slice::Iter<bool> {
        self.bits_boolean.iter()
    }
    /// Constructs from byte length array of bits
    pub fn to_bits<T>(obj: &T, mem: &mut [Bits]){
        let mut bits = mem.iter_mut();
        let size = size_of::<T>();
        let _ptr = obj as *const T;
        let ptr = _ptr as *const u8;
        let bytes = unsafe{from_raw_parts(ptr, size)};
        for byte in bytes.iter(){
            if let Some(bits) = bits.next(){
                *bits = Bits::new(byte);
                continue;
            }
            break;
            
        }
    }
    pub fn from_bits(bits: &mut [u8]) -> Bits {
        bits.reverse();
        let mut val:u8 = 0;
        for (i,b) in bits.iter().enumerate(){
            let mask:u8;
            if *b == 0{
                mask = 0;
            }
            else{
                mask = 0x80;
            }

            val = val | mask;
            if i == bits.len() - 1{
                break;
            }
            val = val >> 1;

        }
        Self::new(&val)
    }
}

impl Default for Bits{
    fn default() -> Self {
        Self{
            src: 0,
            bits_numeric: [0;8],
            bits_boolean: [false;8],
        }
    }
}
