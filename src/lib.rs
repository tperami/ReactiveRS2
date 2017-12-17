#![allow(non_snake_case)]
#![feature(specialization)]
#![feature(log_syntax)]
#![feature(box_syntax, box_patterns)]
#![feature(plugin)]
#![feature(test)]
#![plugin(promacros)]

extern crate core;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate test;


#[macro_use]
pub mod macros;
pub mod engine;
pub mod node;
pub mod process;
pub mod signal;
mod take;


#[cfg(test)]
mod tests {
    use engine::*;
    use process::*;
    use node::*;
    use node::ChoiceData::*;
    use signal::*;
    use test::test::Bencher;
    use std::cell::RefCell;
    use std::rc::*;


    #[test]
    fn instant_action() {
        let mut i = 0;
        {
            let mut r = Runtime::new(mp(|_: ()| { i += 42; }));
            r.execute();
        }
        assert_eq!(i, 42);
    }

    #[test]
    fn sequence() {
        let mut i = 0;
        {
            let mut r = Runtime::new(mp((|_: ()| 42).seq(|v| i = v)));
            r.execute();
        }
        assert_eq!(i, 42);
    }

    #[test]
    fn pause() {
        let mut i = 0;
        let p = &mut i as *mut i32;
        {
            let mut r =
                rt!{
                |_| 42;
                Pause;
                |v| i = v
            };
            r.instant();
            unsafe {
                assert_eq!(*p, 0);
            }
            r.instant();
        }
        assert_eq!(i, 42);
    }

    #[test]
    fn choice() {
        let mut i = 0;
        run!{
            |_| True(42);
            choice {
                |v| i=v
            } {
                |()| unreachable!()
            }
        }
        assert_eq!(i, 42);
    }

    #[test]
    fn choice_pause() {
        let mut i = 0;
        run!{
            |_| True(42);
            Pause;
            choice {
                Pause;
                |v :usize| i = v
            } {
                |()| unreachable!()
            }
        }
        assert_eq!(i, 42);
    }

    #[test]
    fn loop_test() {
        run!{
            |_| 0;
            loop {
                |i : usize| if i < 42 {
                    True(i+1)
                }
                else{
                    False(i)
                };
                Pause
            };
            |i| {
                assert_eq!(i,42)
            }
        }
    }

    #[test]
    fn par() {
        run!{
            |_| (0,0);
            {
                loop {
                    |i : usize|
                    if i < 21 {
                        True(i+1)
                    }
                    else{
                        False(i)
                    };
                    Pause
                } || loop {
                    |i : usize|
                    if i < 21 {
                        True(i+1)
                    }
                    else{
                        False(i)
                    };
                    Pause
                }
            };
            |(v1,v2)| v1 + v2;
            Pause;
            |i| {
                assert_eq!(i,42)
            }
        }
    }

    #[test]
    fn par_half_im() {
        run!{
            |_| (0,0);
            {
                loop {
                    |i : usize|
                    if i < 21 {
                        True(i+1)
                    }
                    else{
                        False(i)
                    };
                    Pause
                } || |_| 21
            };
            |(v1,v2)| v1 + v2;
            Pause;
            |i| {
                assert_eq!(i,42)
            }
        }
    }

    #[test]
    fn par_im() {
        run!{
            |_| (0,0);
            {
                |_| 21 || |_| 21
            };
            |(v1,v2)| v1 + v2;
            Pause;
            |i| {
                assert_eq!(i,42)
            }
        }
    }



    #[test]
    fn emit_await() {
        let mut value = RefCell::new(0);
        let signal = SignalRuntimeRef::new_mc(0, box |e:i32, v:&mut i32| { *v = e;});
        {
            let mut rt = rt! {
                |_| {
                    let signal2 = signal.clone();
                    let signal3 = signal.clone();
                    ((signal2,42), signal3)
                };
                EmitD;
                AwaitD;
                |v| { *value.borrow_mut() = v; }
            };
            rt.instant();
            assert_eq!(*value.borrow_mut(), 0);
            rt.instant();
            assert_eq!(*value.borrow_mut(), 42);
        }
    }

    #[test]
    fn emit_await_immediate() {
        let mut value = RefCell::new(0);
        let signal = SignalRuntimeRef::new_mc(0, box |e:i32, v:&mut i32| { *v = e; });
        {
            let mut rt = rt! {
                |_| {
                    let signal2 = signal.clone();
                    let signal3 = signal.clone();
                    ((signal2,42), signal3)
                };
                EmitD;
                AwaitImmediateD;
                |()| { *value.borrow_mut() = 42; }
            };

            rt.instant();
            assert_eq!(*value.borrow(), 42);
        }
    }


    #[test]
    fn non_await_immediate() {
        let mut value = RefCell::new(0);
        let signal = SignalRuntimeRef::new_mc(0, box |e:i32, v:&mut i32| { *v = e; });
        {
            let mut rt = rt! {
                |_| {
                    let signal3 = signal.clone();
                    signal3
                };
                AwaitImmediateD;
                |()| { *value.borrow_mut() = 42; }
            };

            rt.instant();
            assert_eq!(*value.borrow(), 0);
            rt.instant();
            assert_eq!(*value.borrow(), 0);
        }
    }

    #[test]
    fn emit_pre() {
        let mut value = 0;
        let signal = SignalRuntimeRef::new_mc(1, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    ((signal.clone(),2),((signal.clone(),3),((signal.clone(),7), signal.clone())))
                };
                EmitD;
                EmitD;
                EmitD;
                Pause;
                |val| {
                    value = signal.clone().pre();
                }
            }
        }
        assert_eq!(value, 42);
    }

    #[test]
    fn present_true() {
        let mut value = RefCell::new(0);
        let signal = SignalRuntimeRef::new_pure();
        {
            let mut rt = rt! {
                |_| {
                    ((signal.clone(),()), signal.clone())
                };
                EmitD;
                present
                    {|_:()| {
                        *value.borrow_mut() = 42;
                    }} {
                    |_:()| {
                        *value.borrow_mut() = 21;
                    }}
            };
            rt.instant();
            assert_eq!(*value.borrow_mut(), 42);
        }
    }

    #[test]
    fn present_false() {
        let mut value = RefCell::new(0);
        let signal = SignalRuntimeRef::new_pure();
        {
            let mut rt = rt! {
                |_| {
                    signal.clone()
                };
                present
                    {|_:()| {
                        *value.borrow_mut() = 42;
                    }} {
                    |_:()| {
                        *value.borrow_mut() = 21;
                    }}
            };
            rt.instant();
            assert_eq!(*value.borrow_mut(), 0);
            rt.instant();
            assert_eq!(*value.borrow_mut(), 21);
        }
    }

    #[bench]
    fn bench_emit_pure(bencher: &mut Bencher) {
        bencher.iter(|| {
            let mut r = rt! {
                |_| { ((SignalRuntimeRef::new_pure(),())) };
                EmitD
            };
        r.execute()
        });
    }


    #[bench]
    fn bench_emit_value(bencher: &mut Bencher) {
        bencher.iter(|| {
            let mut r = rt! {
                |_| { ((SignalRuntimeRef::new_mc(1, box |e: i32, v: &mut i32 | { *v *= e; })),42) };
                EmitD
            };
            r.execute();
        });
    }

    #[test]
    fn big_par(){
        let value = Rc::new(RefCell::new(-3));
        {
            let mut processes = vec![];

            for i in 0..10{
                let value2 = value.clone();
                processes.push(pro!{
                    move |_|{
                        *value2.borrow_mut() += i;
                    };
                    Pause
                });
            }
            run!(big_join(processes));
        }
        assert_eq!(*value.borrow(), 42);
    }
}
