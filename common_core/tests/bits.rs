use common_core::bits::Bits;
#[test]
pub fn creation(){
    let tgt:u8 = 0x05;
    let bits = Bits::new(&tgt);
    let bits_from = Bits::from_bits(bits.bits_numeric());
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
