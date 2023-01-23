use common_core::bits::Bits;
#[test]
pub fn creation(){
    let tgt:u8 = 0x05;
    let bits = Bits::new(&tgt);
    let mut _bits = bits.bits_numeric().clone();
    let bits_from = Bits::from_bits(&mut _bits);
    let test_num = [0,0,0,0,0,1,0,1];
    let test_bool = [false,false,false,false,false,true,false,true];
    assert_eq!(test_num, bits.bits_numeric().clone());
    assert_eq!(test_bool, bits.bits_boolean().clone());
    assert_eq!(bits.byte(), bits_from.byte());
}

pub struct Test{
    _byte0: u8,
    _byte1: u8,
    _byte2: u8,
    _byte3: u8,
}
#[test]
pub fn struct_bits(){
    let t = Test{
        _byte0: 0x05,
        _byte1: 0x05,
        _byte2: 0x05,
        _byte3: 0x05,
    };
    let mut bits = [Bits::default();4];
    Bits::to_bits(&t, &mut bits);

    let test_num = [0,0,0,0,0,1,0,1];
    let test_bool = [false,false,false,false,false,true,false,true];

    for bits in bits.iter(){
        assert_eq!(test_num, bits.bits_numeric().clone());
        assert_eq!(test_bool, bits.bits_boolean().clone());
    }
    
}
#[test]
pub fn decompose(){
    let v_addr:u16 = 0x0039;
    let mut v_addr_bits = [Bits::default();2];
    Bits::to_bits(&v_addr, &mut v_addr_bits);
    v_addr_bits.reverse();
    let b1:u8 = 0;
    let b2:u8 = 0x39;
    assert_eq!(b1, v_addr_bits[0].byte());
    assert_eq!(b2, v_addr_bits[1].byte());
}
