#[macro_export]
macro_rules! accioe {
    ($r:ident,$y:ty,$z:ty,$c:ident)
    => 
    {
        super::super::super::acciof!($y,$z,$c);
        let f = AccioF::new($r);
        let ret = f.await;
        ret
    }
}
