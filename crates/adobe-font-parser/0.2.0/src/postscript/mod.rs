mod eexec;
mod parser;
mod vm;

pub use eexec::Decoder;
pub use vm::{RefItem, Vm};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let mut vm = Vm::new();
        vm.parse_and_exec(b"/FontBBox{-180 -293 1090 1010}readonly ")
            .unwrap();
        vm.print_stack();
        assert_eq!(vm.stack().len(), 2);
    }
}
