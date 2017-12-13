use std::marker::PhantomData;

use engine::*;
use signal::*;
use std::rc::Rc;
use std::cell::*;
use std::fmt;
use std::fmt::Debug;

pub trait Node<'a, In: 'a>: 'a {
    type Out: 'a;
    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>, val: In) -> Self::Out;
    fn nseq<N2>(self, n2: N2) -> NSeq<Self, N2>
    where
        N2: Node<'a, Self::Out> + Sized,
        Self: Sized,
    {
        NSeq { n1: self, n2: n2 }
    }
    fn alter<NF, In2: 'a>(self, nf: NF) -> NChoice<Self, NF>
    where
        NF: Node<'a, In2, Out = Self::Out> + Sized,
        Self: Sized,
    {
        NChoice { nt: self, nf: nf }
    }
}



pub struct Nothing {}

impl<'a> Node<'a, ()> for Nothing {
    type Out = ();
    fn call(&mut self, _: &mut SubRuntime<'a>, _val: ()) -> Self::Out {}
}

//  _____                 _
// | ____|_ __ ___  _ __ | |_ _   _
// |  _| | '_ ` _ \| '_ \| __| | | |
// | |___| | | | | | |_) | |_| |_| |
// |_____|_| |_| |_| .__/ \__|\__, |
//                 |_|        |___/

pub struct DummyN<Out> {
    dummy: PhantomData<Out>,
}
impl<Out> Debug for DummyN<Out> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DummyN")
    }
}


impl<'a, In: 'a, Out: 'a> Node<'a, In> for DummyN<Out>
where
    Out: 'a,
{
    type Out = Out;
    fn call(&mut self, _: &mut SubRuntime<'a>, _: In) -> Out {
        panic!("Called empty node");
    }
}


//  _____      __  __       _
// |  ___| __ |  \/  |_   _| |_
// | |_ | '_ \| |\/| | | | | __|
// |  _|| | | | |  | | |_| | |_
// |_|  |_| |_|_|  |_|\__,_|\__|

pub struct FnMutN<F>(pub F);

impl<F> Debug for FnMutN<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Func")
    }
}

impl<'a, F, In: 'a, Out: 'a> Node<'a, In> for FnMutN<F>
where
    F: FnMut(In) -> Out + 'a,
{
    type Out = Out;
    fn call(&mut self, _: &mut SubRuntime<'a>, val: In) -> Out {
        let &mut FnMutN(ref mut f) = self;
        f(val)
    }
}

//  ____
// / ___|  ___  __ _
// \___ \ / _ \/ _` |
//  ___) |  __/ (_| |
// |____/ \___|\__, |
//                |_|

pub struct NSeq<N1, N2> {
    n1: N1,
    n2: N2,
}

impl<'a, N1, N2, In: 'a, Mid: 'a, Out: 'a> Node<'a, In> for NSeq<N1, N2>
where
    N1: Node<'a, In, Out = Mid>,
    N2: Node<'a, Mid, Out = Out>,
{
    type Out = Out;
    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>, val: In) -> Out {
        let valm = self.n1.call(sub_runtime, val);
        self.n2.call(sub_runtime, valm)
    }
}

//  ____        __  __             _
// |  _ \ ___  |  \/  | __ _ _ __ (_)_ __
// | |_) / __| | |\/| |/ _` | '_ \| | '_ \
// |  _ < (__  | |  | | (_| | | | | | |_) |
// |_| \_\___| |_|  |_|\__,_|_| |_|_| .__/
//                                  |_|

type RCell<T> = Rc<Cell<Option<T>>>;

pub fn new_rcell<T>() -> RCell<T> {
    Rc::new(Cell::new(None))
}

pub struct RcStore<T> {
    p: RCell<T>,
}


pub fn store<T>(rc: RCell<T>) -> RcStore<T> {
    RcStore { p: rc }
}

impl<'a, T: 'a> Node<'a, T> for RcStore<T> {
    type Out = ();
    fn call(&mut self, _: &mut SubRuntime<'a>,  val: T) {
        self.p.set(Some(val));
    }
}


pub struct RcLoad<T> {
    p: RCell<T>,
}

pub fn load<T>(rc: RCell<T>) -> RcLoad<T> {
    RcLoad { p: rc }
}



impl<'a, T: 'a> Node<'a, ()> for RcLoad<T> {
    type Out = T;
    fn call(&mut self, _: &mut SubRuntime<'a>, _: ()) -> T {
        self.p.take().unwrap()
    }
}


pub struct RcStoreClone<T> {
    p: RCell<T>,
}

pub fn store_clone<T>(rc: RCell<T>) -> RcStoreClone<T> {
    RcStoreClone {p: rc}
}

impl<'a, T: 'a> Node<'a, T> for RcStoreClone<T>
where
    T: Clone
{
    type Out = T;
    fn call(&mut self, _: &mut SubRuntime<'a>, val: T) -> Self::Out {
        self.p.set(Some(val.clone()));
        val
    }
}

pub struct RcStoreCloneFirst<T> {
    p: RCell<T>,
}

pub fn store_clone_first<T>(rc: RCell<T>) -> RcStoreCloneFirst<T> {
    RcStoreCloneFirst {p: rc}
}

impl<'a, C: 'a, V: 'a> Node<'a, (C,V)> for RcStoreCloneFirst<(C,V)>
where
    C: Clone + 'a,
{
    type Out = C;

    fn call(&mut self, _: &mut SubRuntime<'a>, (clone_val,val): (C,V)) -> Self::Out {
        let out_val = clone_val.clone();
        self.p.set(Some((clone_val,val)));
        out_val
    }
}

//      _
//     | |_   _ _ __ ___  _ __
//  _  | | | | | '_ ` _ \| '_ \
// | |_| | |_| | | | | | | |_) |
//  \___/ \__,_|_| |_| |_| .__/
//                       |_|
pub struct NJump {
    dest: usize,
}

pub fn jump(pos: usize) -> NJump {
    NJump { dest: pos }
}

impl<'a> Node<'a, ()> for NJump {
    type Out = ();
    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>, _: ()) {
        sub_runtime.tasks.current.push(self.dest);
    }
}


//  ____
// |  _ \ __ _ _   _ ___  ___
// | |_) / _` | | | / __|/ _ \
// |  __/ (_| | |_| \__ \  __/
// |_|   \__,_|\__,_|___/\___|

pub struct NPause {
    dest: usize,
}

pub fn pause(pos: usize) -> NPause {
    NPause { dest: pos }
}


impl<'a> Node<'a, ()> for NPause {
    type Out = ();
    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>,  _: ()) {
        sub_runtime.tasks.next.push(self.dest);
    }
}

//   ____ _           _
//  / ___| |__   ___ (_) ___ ___
// | |   | '_ \ / _ \| |/ __/ _ \
// | |___| | | | (_) | | (_|  __/
//  \____|_| |_|\___/|_|\___\___|

pub enum ChoiceData<T, F> {
    True(T),
    False(F),
}
use self::ChoiceData::*;


pub struct NChoice<NT, NF> {
    nt: NT,
    nf: NF,
}

impl<'a,NT,NF, InT: 'a, InF: 'a, Out: 'a> Node<'a, ChoiceData<InT, InF>> for NChoice<NT,NF>
    where
    NT : Node<'a,InT,Out = Out>,
    NF : Node<'a,InF,Out = Out>,
{
    type Out = Out;
    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>,  val: ChoiceData<InT, InF>) -> Out {
        match val {
            True(t) => {
                self.nt.call(sub_runtime, t)
            }
            False(f) => {
                self.nf.call(sub_runtime, f)
            }
        }
    }
}


//  _                     ___
// | |    ___   ___  _ __|_ _|_ __ ___
// | |   / _ \ / _ \| '_ \| || '_ ` _ \
// | |__| (_) | (_) | |_) | || | | | | |
// |_____\___/ \___/| .__/___|_| |_| |_|
//                  |_|

pub struct LoopIm<N>(pub N);

impl<'a, N, In: 'a, Out: 'a> Node<'a, In> for LoopIm<N>
where
    N: Node<'a, In, Out = ChoiceData<In, Out>>,
{
    type Out = Out;
    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>, mut val: In) -> Out {
        let &mut LoopIm(ref mut p) = self;
        loop {
            match p.call(sub_runtime, val) {
                True(t) => {
                    val = t;
                }
                False(f) => {
                    return f;
                }
            }
        }
    }
}


//  _____           _ _
// | ____|_ __ ___ (_) |_
// |  _| | '_ ` _ \| | __|
// | |___| | | | | | | |_
// |_____|_| |_| |_|_|\__|

#[derive(Clone, Copy)]
pub struct NEmitD {}

impl<'a, SV: 'a, E: 'a, In: 'a> Node<'a, ((SignalRuntimeRef<SV>, E), In)> for NEmitD
where
    SV: SignalValue<E=E>,
{
    type Out = In;

    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>, ((sr,e),val): ((SignalRuntimeRef<SV>, E), In)) -> Self::Out {
        sr.emit(e, sub_runtime);
        val
    }
}

impl<'a, SV: 'a, E: 'a> Node<'a, (SignalRuntimeRef<SV>, E)> for NEmitD
where
    SV: SignalValue<E=E>,
{
    type Out = ();

    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>, (sr,e): (SignalRuntimeRef<SV>, E)) -> () {
        sr.emit(e, sub_runtime);
        ()
    }
}



//   ____      _
//  / ___| ___| |_
// | |  _ / _ \ __|
// | |_| |  __/ |_
//  \____|\___|\__|


#[derive(Clone, Copy)]
pub(crate) struct NGetD {}

impl<'a, SV: 'a, V: 'a, In: 'a> Node<'a, (SignalRuntimeRef<SV>, In)> for NGetD
where
    SV: SignalValue<V=V>,
{
    type Out = (V,In);

    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>, (sr,val): (SignalRuntimeRef<SV>, In)) -> Self::Out {
        (sr.signal_runtime.values.get_pre_value(), val)
    }
}

impl<'a, SV: 'a, V: 'a> Node<'a, SignalRuntimeRef<SV>> for NGetD
where
    SV: SignalValue<V=V>,
{
    type Out = V;

    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>, sr: SignalRuntimeRef<SV>) -> Self::Out {
        sr.signal_runtime.values.get_pre_value()
    }
}




// __        __    _ _
// \ \      / /_ _(_) |_
//  \ \ /\ / / _` | | __|
//   \ V  V / (_| | | |_
//    \_/\_/ \__,_|_|\__|



#[derive(Clone,Copy)]
pub(crate) struct NWaitD(pub usize);

impl<'a, SV: 'a, In: 'a> Node<'a, (SignalRuntimeRef<SV>, In)> for NWaitD
where
    SV: SignalValue,
{
    type Out = In;

    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>, (sr,val): (SignalRuntimeRef<SV>, In)) -> Self::Out {
        sr.on_signal(&mut sub_runtime.tasks, self.0);
        val
    }
}

impl<'a, SV: 'a> Node<'a, SignalRuntimeRef<SV>> for NWaitD
where
    SV: SignalValue,
{
    type Out = ();

    fn call(&mut self, sub_runtime: &mut SubRuntime<'a>, sr: SignalRuntimeRef<SV>) -> Self::Out {
        sr.on_signal(&mut sub_runtime.tasks, self.0);
    }
}