use abridgment::prelude::*;

#[test]
fn floor() {
    assert_eq!(0.0, abridge::<0>(0.123456789, Mantissa, Floor));
    assert_eq!(0.1, abridge::<1>(0.123456789, Mantissa, Floor));
    assert_eq!(0.12, abridge::<2>(0.123456789, Mantissa, Floor));
    assert_eq!(0.123, abridge::<3>(0.123456789, Mantissa, Floor));
    assert_eq!(0.1234, abridge::<4>(0.123456789, Mantissa, Floor));
    assert_eq!(0.12345, abridge::<5>(0.123456789, Mantissa, Floor));
    assert_eq!(0.123456, abridge::<6>(0.123456789, Mantissa, Floor));
    assert_eq!(0.1234567, abridge::<7>(0.123456789, Mantissa, Floor));
    assert_eq!(0.12345678, abridge::<8>(0.123456789, Mantissa, Floor));
    assert_eq!(0.123456789, abridge::<9>(0.123456789, Mantissa, Floor));
}

#[test]
fn ceil() {
    assert_eq!(1.0, abridge::<0>(0.123456789, Mantissa, Ceil));
    assert_eq!(0.2, abridge::<1>(0.123456789, Mantissa, Ceil));
    assert_eq!(0.13, abridge::<2>(0.123456789, Mantissa, Ceil));
    assert_eq!(0.124, abridge::<3>(0.123456789, Mantissa, Ceil));
    assert_eq!(0.1235, abridge::<4>(0.123456789, Mantissa, Ceil));
    assert_eq!(0.12346, abridge::<5>(0.123456789, Mantissa, Ceil));
    assert_eq!(0.123457, abridge::<6>(0.123456789, Mantissa, Ceil));
    assert_eq!(0.1234568, abridge::<7>(0.123456789, Mantissa, Ceil));
    assert_eq!(0.12345679, abridge::<8>(0.123456789, Mantissa, Ceil));
    assert_eq!(0.123456789, abridge::<9>(0.123456789, Mantissa, Ceil));
}

#[test]
fn round() {
    assert_eq!(0.0, abridge::<0>(0.123456789, Mantissa, Round));
    assert_eq!(0.1, abridge::<1>(0.123456789, Mantissa, Round));
    assert_eq!(0.12, abridge::<2>(0.123456789, Mantissa, Round));
    assert_eq!(0.123, abridge::<3>(0.123456789, Mantissa, Round));
    assert_eq!(0.1235, abridge::<4>(0.123456789, Mantissa, Round));
    assert_eq!(0.12346, abridge::<5>(0.123456789, Mantissa, Round));
    assert_eq!(0.123457, abridge::<6>(0.123456789, Mantissa, Round));
    assert_eq!(0.1234568, abridge::<7>(0.123456789, Mantissa, Round));
    assert_eq!(0.12345679, abridge::<8>(0.123456789, Mantissa, Round));
    assert_eq!(0.123456789, abridge::<9>(0.123456789, Mantissa, Round));
}
