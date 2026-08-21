
#[macro_export]
macro_rules! acciof {
    ($y:ty,$z:ty,$c:ident) => {
        $crate::acciod!($c);
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
    }
}

#[macro_export]
macro_rules! acciod {
    ($c:ident) => {
        unsafe extern "C" {
            fn $c(_: *const (), _: *const (), _: *const ());
        }
    };
}

#[macro_export]
macro_rules! acciob {
    ($x:ident,$p:ty,$q:ty,$c:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $c(rsp: *const (), resp: *const ()) {
            let resp = resp as *mut () as *mut $x;
            let rsp = rsp as *const $q;
            let rsp = unsafe {&*rsp};
            let x: $p = rsp.into();
            let resp = unsafe { &mut *resp};
            resp.resp = Some(x);
            resp.st = 2;
            if let Some(wk) = resp.wk.take() {
                wk.wake();
            }
        }
    };
}

#[macro_export]
macro_rules! accioe {
    ($r:ident,$y:ty,$z:ty,$q:ty,$c:ident,$b:ident)
    => 
    {
        {
            let r: $y = $r.into();
            $crate::acciof!($y,$z,$c);
            $crate::acciob!(AccioF,$z,$q,$b);
            let f = AccioF::new(r);
            let ret = f.await;
            ret
        }
    }
}
