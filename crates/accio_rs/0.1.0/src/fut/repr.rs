
#[macro_export]
macro_rules! acciof {
    ($y:ty,$z:ty,$c:ident) => {
        $crate::acciof!(AccioF,$y,$z,$c)
    };
    ($x:ident,$y:ty,$z:ty,$c:ident) => {
        #[repr(C)]
        struct $x {
            req: $y,
            st: i8,
            resp: Option<$z>,
            wk: Option<std::task::Waker>,
        }
        impl $x {
            fn new(r: $y) -> Self {
                Self {
                    req: r,
                    st: 0,
                    resp: None,
                    wk: None,
                }
            }
        }
        unsafe impl Send for $x {}
        unsafe impl Sync for $x {}
        impl futures::Future for $x {
            type Output = Option<$z>;
            fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
                let f = self.get_mut();
                match f.st {
                    0 => {
                        let wk = cx.waker().clone();
                        f.wk = Some(wk);
                        f.st = 1;
                        #[allow(unused_unsafe)]
                        unsafe { $c(&f.req as *const _ as *const (), &f.req as *const _ as *const (), f as *mut _ as *const _); };
                    },
                    1 => {},
                    2 => { return std::task::Poll::Ready(f.resp.take()); },
                    _ => { panic!("not ok")},
                }
                return std::task::Poll::Pending;
            }
        }
    };
}
