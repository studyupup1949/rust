pub fn add() {
    macro_rules! addtwo {
        ($value:expr) => {
            $value * 2
        };
    }
}
