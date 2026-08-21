#[macro_export]
macro_rules! accioe {
    ($r:ident,$y:ty,$z:ty,$q:ty,$c:ident,$b:ident)
    => 
    {
        {
            let r: $y = $r.into();
            super::super::super::acciof!($y,$z,$c);
            super::super::super::acciob!(AccioF,$z,$q,$b);
            let f = AccioF::new(r);
            let ret = f.await;
            ret
        }
    }
}
