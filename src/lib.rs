#![allow(non_snake_case)]
#![feature(specialization)]
#![feature(log_syntax)]
#![feature(box_syntax, box_patterns)]
#![feature(plugin)]
#![feature(test)]
#![plugin(promacros)]
#![feature(core_intrinsics)]
#![feature(arbitrary_self_types)]
#![feature(conservative_impl_trait)]
#![feature(associated_type_defaults)]

// #[macro_use] extern crate log;
// extern crate env_logger;
extern crate test;


#[macro_use]
pub mod macros;
pub mod engine;
pub mod node;
pub mod process;
pub mod signal;
mod take;
mod tname;

#[cfg(test)]
mod tests {
    use engine::*;

    use process::*;
    use process::pause;
    use node::ChoiceData::*;
    use signal::*;
    use test::test::Bencher;
    use std::cell::RefCell;
    use std::rc::*;


    #[test]
    fn instant_action_no_macro() {
        let mut i = 0;
        {
            let mut r = Runtime::new(fnmut2pro(|_| { i += 42; }));
            r.execute();
        }
        assert_eq!(i, 42);
    }

    #[test]
    fn instant_action(){
        let mut i = 0;
        run!(|_| { i += 42; });
        assert_eq!(i, 42);
    }


    #[test]
    fn sequence() {
        let mut i = 0;
        run!{|_ :()| 42;
             |v : usize| i = v;
        };
        assert_eq!(i, 42);
    }

    #[test]
    fn pauset() {
        let mut i = 0;
        let p = &mut i as *mut i32;
        {
            let mut r = rt!{
                |_| 42;
                pause();
                |v| i = v;
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
            pause();
            choice {
                pause();
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
                pause()
            };
            |i| {
                assert_eq!(i,42)
            }
        }
    }

    #[test]
    fn emit_d_test() {
        let mut value = 0;
        let signal = SignalRuntimeRef::new_mc(21, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    (signal.clone(),2)
                };
                emit_d();
                pause();
                |_| {
                    value = signal.clone().pre();
                }
            }
        }
        assert_eq!(value, 42);
    }

    #[test]
    fn emit_d_in_test() {
        let mut value = 0;
        let signal = SignalRuntimeRef::new_mc(7, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    ((signal.clone(),2), 3)
                };
                emit_d_in();
                pause();
                |val| {
                    value = val * signal.clone().pre();
                }
            }
        }
        assert_eq!(value, 42);
    }

    #[test]
    fn emit_d_vec_test() {
        let mut value = 0;
        let signal = SignalRuntimeRef::new_mc(7, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    vec![(signal.clone(),2), (signal.clone(), 3)]
                };
                emit_d_vec();
                pause();
                |_| {
                    value = signal.clone().pre();
                }
            }
        }
        assert_eq!(value, 42);
    }

    #[test]
    fn emit_d_vec_in_test() {
        let mut value = 0;
        let signal = SignalRuntimeRef::new_mc(1, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    (vec![(signal.clone(),2), (signal.clone(), 3)], 7)
                };
                emit_d_vec_in();
                pause();
                |val| {
                    value = val * signal.clone().pre();
                }
            }
        }
        assert_eq!(value, 42);
    }

    #[test]
    fn emit_s_test() {
        let mut value = 0;
        let signal = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    21
                };
                emit_s(signal.clone());
                pause();
                |_| {
                    value = signal.clone().pre();
                }
            }
        }
        assert_eq!(value, 42);
    }

    #[test]
    fn emit_s_in_test() {
        let mut value = 0;
        let signal = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    (7,3)
                };
                emit_s_in(signal.clone());
                pause();
                |val| {
                    value = val * signal.clone().pre();
                }
            }
        }
        assert_eq!(value, 42);
    }


    #[test]
    fn emit_vec_s_test() {
        let mut value = 0;
        let signal1 = SignalRuntimeRef::new_mc(1, box |e: i32, v: &mut i32| { *v *= e;});
        let signal2 = SignalRuntimeRef::new_mc(1, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    vec![2,14]
                };
                emit_vec_s(vec![signal1.clone(), signal2.clone()]);
                pause();
                |_:()| {
                    value = 7 * signal1.clone().pre() + 2 * signal2.clone().pre();
                }
            }
        }
        assert_eq!(value, 42);
    }

    #[test]
    fn emit_vec_s_in_test() {
        let mut value = 0;
        let signal1 = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        let signal2 = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    (vec![2,14], 2)
                };
                emit_vec_s_in(vec![signal1.clone(), signal2.clone()]);
                pause();
                |val| {
                    value = (7 * signal1.clone().pre() + 2 * signal2.clone().pre()) / val;
                }
            }
        }
        assert_eq!(value, 42);
    }

    #[test]
    fn emit_vs_test() {
        let mut value = 0;
        let signal1 = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    ()
                };
                emit_vs(signal1.clone(), 21);
                pause();
                |_| {
                    value = signal1.clone().pre();
                }
            }
        }
        assert_eq!(value, 42);
    }

    #[test]
    fn emit_vec_vs_test() {
        let mut value = 0;
        let signal1 = SignalRuntimeRef::new_mc(1, box |e: i32, v: &mut i32| { *v *= e;});
        let signal2 = SignalRuntimeRef::new_mc(1, box |e: i32, v: &mut i32| { *v *= e;});
        {
            run! {
                |_| {
                    7
                };
                emit_vec_vs(vec![(signal1.clone(), 2), (signal2.clone(), 3)]);
                pause();
                |val| {
                    value = val * signal1.clone().pre() * signal2.clone().pre();
                }
            }
        }
        assert_eq!(value, 42);
    }


    #[test]
    fn await_d_test() {
        let value = Rc::new(RefCell::new(0));
        let signal = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            let mut rt = rt! {
                |_| {
                    signal.clone()
                };
                emit_vs(signal.clone(), 21);
                await_d();
                |val| {
                    *value.borrow_mut() = val
                }
            };

            rt.instant();
            assert_eq!(*value.borrow(), 0);
            rt.instant();
            assert_eq!(*value.borrow(), 42);
        }
    }

    #[test]
    fn non_await_d_test() {
        let value = Rc::new(RefCell::new(0));
        let signal = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            let mut rt = rt! {
                |_| {
                    signal.clone()
                };
                await_d();
                |val| {
                    *value.borrow_mut() = val
                }
            };

            rt.instant();
            assert_eq!(*value.borrow(), 0);
            rt.instant();
            assert_eq!(*value.borrow(), 0);
        }
    }

    #[test]
    fn await_d_in_test() {
        let value = Rc::new(RefCell::new(0));
        let signal = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            let mut rt = rt! {
                |_| {
                    (signal.clone(), 7)
                };
                emit_vs(signal.clone(), 3);
                await_d_in();
                |(val1, val2)| {
                    *value.borrow_mut() = val1 * val2
                }
            };

            rt.instant();
            assert_eq!(*value.borrow(), 0);
            rt.instant();
            assert_eq!(*value.borrow(), 42);
        }
    }

    #[test]
    fn non_await_d_in_test() {
        let value = Rc::new(RefCell::new(0));
        let signal = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            let mut rt = rt! {
                |_| {
                    (signal.clone(), 7)
                };
                await_d_in();
                |(val1, val2)| {
                    *value.borrow_mut() = val1 * val2
                }
            };

            rt.instant();
            assert_eq!(*value.borrow(), 0);
            rt.instant();
            assert_eq!(*value.borrow(), 0);
        }
    }

    #[test]
    fn await_s_test() {
        let value = Rc::new(RefCell::new(0));
        let signal = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            let mut rt = rt! {
                |_| {
                    ()
                };
                emit_vs(signal.clone(), 21);
                await_s(signal.clone());
                |val| {
                    *value.borrow_mut() = val
                }
            };

            rt.instant();
            assert_eq!(*value.borrow(), 0);
            rt.instant();
            assert_eq!(*value.borrow(), 42);
        }
    }

    #[test]
    fn non_await_s_test() {
        let value = Rc::new(RefCell::new(0));
        let signal = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            let mut rt = rt! {
                |_| {
                    ()
                };
                await_s(signal.clone());
                |val| {
                    *value.borrow_mut() = val
                }
            };

            rt.instant();
            assert_eq!(*value.borrow(), 0);
            rt.instant();
            assert_eq!(*value.borrow(), 0);
        }
    }

    #[test]
    fn await_s_in_test() {
        let value = Rc::new(RefCell::new(0));
        let signal = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            let mut rt = rt! {
                |_| {
                    7
                };
                emit_vs(signal.clone(), 3);
                await_s_in(signal.clone());
                |(val1, val2)| {
                    *value.borrow_mut() = val1 * val2
                }
            };

            rt.instant();
            assert_eq!(*value.borrow(), 0);
            rt.instant();
            assert_eq!(*value.borrow(), 42);
        }
    }

    #[test]
    fn non_await_s_in_test() {
        let value = Rc::new(RefCell::new(0));
        let signal = SignalRuntimeRef::new_mc(2, box |e: i32, v: &mut i32| { *v *= e;});
        {
            let mut rt = rt! {
                |_| {
                    7
                };
                await_s_in(signal.clone());
                |(val1, val2)| {
                    *value.borrow_mut() = val1 * val2
                }
            };

            rt.instant();
            assert_eq!(*value.borrow(), 0);
            rt.instant();
            assert_eq!(*value.borrow(), 0);
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
                    pause()
                } || loop {
                    |i : usize|
                    if i < 21 {
                        True(i+1)
                    }
                    else{
                        False(i)
                    };
                    pause()
                }
            };
            |(v1,v2)| v1 + v2;
            pause();
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
                    pause()
                } || |_| 21
            };
            |(v1,v2)| v1 + v2;
            pause();
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
            pause();
            |i| {
                assert_eq!(i,42)
            }
        }
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
                    pause()
                });
            }
            run!(big_join(processes));
        }
        assert_eq!(*value.borrow(), 42);
    }


    #[bench]
    fn bench_emitd_pause(bencher: &mut Bencher) {
        let signal = SignalRuntimeRef::new_pure();
        let mut rt = rt! {
            loop {
                |_| { (signal.clone(), ()) };
                emit_d();
                pause();
                |_| {
                    True(())
                }
            }
        };

        bencher.iter(|| {
            for _ in 0..1000 {
                rt.instant();
            }
        });
    }


    #[bench]
    fn bench_emits_pause(bencher: &mut Bencher) {
        let signal = SignalRuntimeRef::new_pure();
        let mut rt = rt! {
            loop {
                |_:()| { };
                emit_vs(signal.clone(),());
                pause();
                |_| {
                    True(())
                }
            }
        };

        bencher.iter(|| {
            for _ in 0..1000 {
                rt.instant();
            }
        });
    }


    #[bench]
    fn bench_emitd_await(bencher: &mut Bencher) {
        let signal = SignalRuntimeRef::new_pure();
        let mut rt = rt! {
            loop {
                |_| { ((signal.clone(), ()), signal.clone()) };
                emit_d_in();
                await_d();
                |_:()| {
                    True(())
                }
            }
        };

        bencher.iter(|| {
            for _ in 0..1000 {
                rt.instant();
            }
        });
    }


    #[bench]
    fn bench_emits_await(bencher: &mut Bencher) {
        let signal = SignalRuntimeRef::new_pure();
        let mut rt = rt! {
            loop {
                |_| {()};
                emit_vs(signal.clone(), ());
                await_s(signal.clone());
                |_:()| {
                    True(())
                }
            }
        };

        bencher.iter(|| {
            for _ in 0..1000 {
                rt.instant();
            }
        });
    }
}
