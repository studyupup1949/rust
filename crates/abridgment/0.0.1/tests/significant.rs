use abridgment::prelude::*;

#[test]
fn floor() {
    assert_eq!(0.0, abridge::<0>(0.0123456789, Significant, Floor));
    assert_eq!(0.01, abridge::<1>(0.0123456789, Significant, Floor));
    assert_eq!(0.012, abridge::<2>(0.0123456789, Significant, Floor));
    assert_eq!(0.0123, abridge::<3>(0.0123456789, Significant, Floor));
    assert_eq!(0.01234, abridge::<4>(0.0123456789, Significant, Floor));
    assert_eq!(0.012345, abridge::<5>(0.0123456789, Significant, Floor));
    assert_eq!(0.0123456, abridge::<6>(0.0123456789, Significant, Floor));
    assert_eq!(0.01234567, abridge::<7>(0.0123456789, Significant, Floor));
    assert_eq!(0.012345678, abridge::<8>(0.0123456789, Significant, Floor));
    assert_eq!(0.0123456789, abridge::<9>(0.0123456789, Significant, Floor));
}

#[test]
fn ceil() {
    assert_eq!(0.10, abridge::<0>(0.0123456789, Significant, Ceil));
    assert_eq!(0.02, abridge::<1>(0.0123456789, Significant, Ceil));
    assert_eq!(0.013, abridge::<2>(0.0123456789, Significant, Ceil));
    assert_eq!(0.0124, abridge::<3>(0.0123456789, Significant, Ceil));
    assert_eq!(0.01235, abridge::<4>(0.0123456789, Significant, Ceil));
    assert_eq!(0.012346, abridge::<5>(0.0123456789, Significant, Ceil));
    assert_eq!(0.0123457, abridge::<6>(0.0123456789, Significant, Ceil));
    assert_eq!(0.01234568, abridge::<7>(0.0123456789, Significant, Ceil));
    assert_eq!(0.012345679, abridge::<8>(0.0123456789, Significant, Ceil));
    assert_eq!(0.0123456789, abridge::<9>(0.0123456789, Significant, Ceil));
}

#[test]
fn round() {
    assert_eq!(0.0, abridge::<0>(0.0123456789, Significant, Round));
    assert_eq!(0.01, abridge::<1>(0.0123456789, Significant, Round));
    assert_eq!(0.012, abridge::<2>(0.0123456789, Significant, Round));
    assert_eq!(0.0123, abridge::<3>(0.0123456789, Significant, Round));
    assert_eq!(0.01235, abridge::<4>(0.0123456789, Significant, Round));
    assert_eq!(0.012346, abridge::<5>(0.0123456789, Significant, Round));
    assert_eq!(0.0123457, abridge::<6>(0.0123456789, Significant, Round));
    assert_eq!(0.01234568, abridge::<7>(0.0123456789, Significant, Round));
    assert_eq!(0.012345679, abridge::<8>(0.0123456789, Significant, Round));
    assert_eq!(0.0123456789, abridge::<9>(0.0123456789, Significant, Round));
}
