use crate::link::Link;

pub trait Page {
    fn part_of(&self) -> &Link;
    fn next(&self) -> &Link;
    fn prev(&self) -> &Link;
}
