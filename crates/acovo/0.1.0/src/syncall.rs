
/// $r: return value, $s: success value, $e:action , $t: timeout in seconds
#[cfg(feature = "syncall")]
#[macro_export]
macro_rules! syncall_with_timeout {
    ($r:expr,$s:expr,$e:expr,$t:expr) => {
        let tm_start = Instant::now();
        while (tm_start.elapsed().as_secs() <= $t as u64) {
            let ret = $e;
            if ret.is_ok() {
                $r = ret;
                break;
            } else {
                continue;
            }
        }
    };
}

/// $r: return value, $s: exit signal, $e:action , $t: timeout in seconds, $v: retry interval
#[cfg(feature = "syncall")]
#[macro_export]
macro_rules! syncall_with_signal_timeout {
    ($r:expr,$s:expr,$e:expr,$t:expr,$v:expr) => {
        //init lock flag.
        unsafe {
            let mut value = $s.lock().unwrap();
            *value = false;
            debug!("sync_call run_flag init");
            //Mutex::unlock(value);
        }

        let tm_start = Instant::now();

        while tm_start.elapsed().as_secs() <= $t as u64 {
            $r = $e;

            let lock = unsafe { $s.lock().unwrap() };
            let exit_flag = *lock;
            //Mutex::unlock(lock);

            if $r.is_ok() || exit_flag {
                debug!("sync_call break");
                break;
            }
            //debug!("sync_call run_flag={}", run_flag);
            thread::sleep(Duration::from_secs($v));
        }
        debug!("sync_call exit");
    };
}


#[cfg(feature = "syncall")]
#[macro_export]
macro_rules! atomic_call_imt {
    ($r:expr,$l:expr,$b:ident) => {
        unsafe {
            let guard = &$l;
            let mut retry = 3;
            while retry > 0 {
                match guard.try_borrow() {
                    Ok(v) => {
                        $r = v.$b();
                        retry = 0;
                    }
                    Err(e) => {
                        error!("BorrowFailed {}", retry);
                        retry -= 1;
                    }
                }
            }
        };
    };
}

#[cfg(feature = "syncall")]
#[macro_export]
macro_rules! atomic_call {
    ($r:expr,$l:expr,$b:ident) => {
        unsafe {
            let guard = &$l;
            let mut retry = 3;
            while retry > 0 {
                match guard.try_borrow_mut() {
                    Ok(mut v) => {
                        $r = v.$b();
                        retry = 0;
                    }
                    Err(e) => {
                        error!("BorrowFailed {}", retry);
                        retry -= 1;
                    }
                }
            }
        };
    };
}

#[cfg(feature = "syncall")]
#[macro_export]
macro_rules! atomic_call_arg1 {
    ($r:expr,$l:expr,$b:ident,$a:expr) => {
        unsafe {
            let guard = &$l;
            let mut retry = 3;
            while retry > 0 {
                match guard.try_borrow_mut() {
                    Ok(mut v) => {
                        $r = v.$b($a);
                        retry = 0;
                    }
                    Err(e) => {
                        error!("BorrowFailed {}", retry);
                        retry -= 1;
                    }
                }
            }
        }
    };
}

#[cfg(feature = "syncall")]
#[macro_export]
macro_rules! atomic_call_arg2 {
    ($ret:expr,$l:expr,$f:ident,$a:expr,$b:expr) => {
        unsafe {
            let guard = &$l;
            let mut retry = 3;
            while retry > 0 {
                match guard.try_borrow_mut() {
                    Ok(mut v) => {
                        $ret = v.$f($a, $b);
                        retry = 0;
                    }
                    Err(e) => {
                        error!("BorrowFailed {}", retry);
                        retry -= 1;
                    }
                }
            }
        };
    };
}

#[cfg(feature = "syncall")]
#[macro_export]
macro_rules! atomic_call_arg3 {
    ($ret:expr,$l:expr,$f:ident,$a:expr,$b:expr,$c:expr) => {
        unsafe {
            let guard = &$l;
            let mut retry = 3;
            while retry > 0 {
                match guard.try_borrow_mut() {
                    Ok(mut v) => {
                        $ret = v.$f($a, $b, $c);
                        retry = 0;
                    }
                    Err(e) => {
                        error!("BorrowFailed {}", retry);
                        retry -= 1;
                    }
                }
            }
        };
    };
}

#[cfg(feature = "syncall")]
#[macro_export]
macro_rules! atomic_call_arg4 {
    ($ret:expr,$l:expr,$f:ident,$a:expr,$b:expr,$c:expr,$d:expr) => {
        unsafe {
            let guard = &$l;
            let mut retry = 3;
            while retry > 0 {
                match guard.try_borrow_mut() {
                    Ok(mut v) => {
                        $ret = v.$f($a, $b, $c, $d);
                        retry = 0;
                    }
                    Err(e) => {
                        error!("BorrowFailed {}", retry);
                        retry -= 1;
                    }
                }
            }
        };
    };
}

#[cfg(feature = "syncall")]
#[macro_export]
macro_rules! state_call {
    ($r:expr,$l:expr,$b:ident) => {
        unsafe {
            let mut retry = 3;
            while retry > 0 {
                match $l.get().write() {
                    Ok(mut v) => {
                        $r = v.$b();
                        retry = 0;
                    }
                    Err(e) => {
                        error!("BorrowFailed {}", retry);
                        retry -= 1;
                    }
                }
            }
        };
    };
}

#[cfg(feature = "syncall")]
#[macro_export]
macro_rules! state_call_imt {
    ($r:expr,$l:expr,$b:ident) => {
        unsafe {
            let mut retry = 3;
            while retry > 0 {
                match $l.get().read() {
                    Ok(v) => {
                        $r = v.$b();
                        retry = 0;
                    }
                    Err(e) => {
                        error!("BorrowFailed {}", retry);
                        retry -= 1;
                    }
                }
            }
        };
    };
}

#[cfg(test)]
#[cfg(feature = "syncall")]
mod tests {

    use super::*;
    use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
    use crossbeam_utils::thread as CrossThread;
    use std::cell::LazyCell;
    use std::sync::Mutex;
    use std::time::Duration;
    use std::time::Instant;
    use tracing::error;
    use tracing::debug;
    use std::{thread, time};
    use anyhow::Result as AnyResult;

    static mut SignalExit: Mutex<bool> = Mutex::new(false);

    pub static mut CBK_MMT: LazyCell<AtomicRefCell<Box<dyn MutableTrait>>> = LazyCell::new(|| {
        AtomicRefCell::new(Box::new(MutableSystem {
            info: "".to_string(),
        }))
    });

    pub trait MutableTrait {
        fn test(&mut self);
        fn test1(&mut self,f:u8)->u8;
        fn test2(&mut self,f:u8,a:u8)->String;
        fn test3(&mut self,f:u8,a:u8,b:u8)->bool;
        fn test4(&mut self,f:u8,a:u8,b:u8,c:u8)->bool;
    }

    pub struct MutableSystem {
        pub info: String,
    }

    impl MutableTrait for MutableSystem {
        fn test(&mut self) {
            self.info = "X".to_string();
            println!(".")
        }

        fn test1(&mut self, f: u8) -> u8 {
            self.info = "X".to_string();
            println!("t1");
            1
        }

        fn test2(&mut self, f: u8, a: u8) -> String {
            self.info = "X".to_string();
            println!("t2");
            "".to_string()
        }

        fn test3(&mut self, f: u8, a: u8, b: u8) -> bool {
            self.info = "X".to_string();
            println!("t3");
            true
        }
        fn test4(&mut self, f: u8, a: u8, b: u8,c:u8) -> bool {
            self.info = "X".to_string();
            println!("t3");
            true
        }
    }

    fn sleep_test(timeout: u8, info: &mut u8) -> AnyResult<u8> {
        println!("sleep_test {}", timeout);
        thread::sleep(Duration::from_secs(1));
        *info = 3 as u8;
        Ok(1)
    }

    #[test]
    fn test_syncall() {
        thread::spawn(|| loop {
            thread::sleep(Duration::from_secs(8));
            unsafe {
                let mut value = SignalExit.lock().unwrap();
                *value = true;
            }
        });

        let mut ret = Ok(0);
        let mut ifo = 2;
        let s = 1;
        syncall_with_timeout!(ret, 1, sleep_test(s, &mut ifo), 10);
        println!("{:?}", ret);

        syncall_with_signal_timeout!(ret, SignalExit, sleep_test(s, &mut ifo), 10,1);
        println!("{:?}", ret);
    }

    #[test]
    fn test_atomic_call() {
        let mut timer = Instant::now();
        CrossThread::scope(|scope| {

            for _ in 0..10000 {
                scope.spawn(|_| loop {

                        let mut ret1 = 0;
                        atomic_call!(CBK_MMT, test);
                        atomic_call_arg1!(ret1, CBK_MMT, test1, 1);
                        println!("test1 ret={:?}", ret1);
                        let mut ret2 = "".to_string();
                        atomic_call_arg2!(ret2, CBK_MMT, test2, 1, 2);
                        println!("test2 ret={:?}", ret2);
                        let mut ret3 = false;
                        atomic_call_arg3!(ret3, CBK_MMT, test3, 1, 2, 3);
                        println!("test3 ret={:?}", ret3);
                        let mut ret4 = false;
                        atomic_call_arg4!(ret4, CBK_MMT, test4, 1, 2, 3,4);
                        println!("test4 ret={:?}", ret4);
                        return;
                });
            }
            
        }).unwrap();

        println!("elapsed {:?}", timer.elapsed());
    }
}
