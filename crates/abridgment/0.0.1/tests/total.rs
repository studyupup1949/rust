use abridgment::prelude::*;

#[test]
fn floor() {
    assert_eq!(1234.0, abridge::<0>(1234.56789, Total, Floor));
    assert_eq!(1234.0, abridge::<1>(1234.56789, Total, Floor));
    assert_eq!(1234.0, abridge::<2>(1234.56789, Total, Floor));
    assert_eq!(1234.0, abridge::<3>(1234.56789, Total, Floor));
    assert_eq!(1234.0, abridge::<4>(1234.56789, Total, Floor));
    assert_eq!(1234.5, abridge::<5>(1234.56789, Total, Floor));
    assert_eq!(1234.56, abridge::<6>(1234.56789, Total, Floor));
    assert_eq!(1234.567, abridge::<7>(1234.56789, Total, Floor));
    assert_eq!(1234.5678, abridge::<8>(1234.56789, Total, Floor));
    assert_eq!(1234.56789, abridge::<9>(1234.56789, Total, Floor));
}

#[test]
fn ceil() {
    assert_eq!(1235.0, abridge::<0>(1234.56789, Total, Ceil));
    assert_eq!(1235.0, abridge::<1>(1234.56789, Total, Ceil));
    assert_eq!(1235.0, abridge::<2>(1234.56789, Total, Ceil));
    assert_eq!(1235.0, abridge::<3>(1234.56789, Total, Ceil));
    assert_eq!(1235.0, abridge::<4>(1234.56789, Total, Ceil));
    assert_eq!(1234.6, abridge::<5>(1234.56789, Total, Ceil));
    assert_eq!(1234.57, abridge::<6>(1234.56789, Total, Ceil));
    assert_eq!(1234.568, abridge::<7>(1234.56789, Total, Ceil));
    assert_eq!(1234.5679, abridge::<8>(1234.56789, Total, Ceil));
    assert_eq!(1234.56789, abridge::<9>(1234.56789, Total, Ceil));
}

#[test]
fn round() {
    assert_eq!(1235.0, abridge::<0>(1234.56789, Total, Round));
    assert_eq!(1235.0, abridge::<1>(1234.56789, Total, Round));
    assert_eq!(1235.0, abridge::<2>(1234.56789, Total, Round));
    assert_eq!(1235.0, abridge::<3>(1234.56789, Total, Round));
    assert_eq!(1235.0, abridge::<4>(1234.56789, Total, Round));
    assert_eq!(1234.6, abridge::<5>(1234.56789, Total, Round));
    assert_eq!(1234.57, abridge::<6>(1234.56789, Total, Round));
    assert_eq!(1234.568, abridge::<7>(1234.56789, Total, Round));
    assert_eq!(1234.5679, abridge::<8>(1234.56789, Total, Round));
    assert_eq!(1234.56789, abridge::<9>(1234.56789, Total, Round));
}
