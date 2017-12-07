use node::*;
use engine::*;
use std::marker::PhantomData;
use std::rc::Rc;
use std::cell::*;

pub trait Is {
    type Value;
}

impl<T> Is for T {
    type Value = T;
}

pub struct NotIm {}
pub struct IsIm {}
pub trait Im: Sized {}
impl Im for NotIm {}
impl Im for IsIm {}


pub trait Process<'a, In: 'a>: 'a + Sized {
    type Out: 'a;
    type NI: Node<'a, In, Out = ()> + Sized;
    type NO: Node<'a, (), Out = Self::Out> + Sized;
    /// If mark is set to IsIm, compile panics, if it is NotIm, compileIm panics
    type Mark: Im;
    fn compile(self, _: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        unreachable!();
    }
    type NIO: Node<'a, In, Out = Self::Out>;
    fn compileIm(self, _: &mut Graph<'a>) -> Self::NIO {
        unreachable!();
    }
    fn seq<P>(self, p: P) -> Seq<MarkedProcess<Self, Self::Mark>, MarkedProcess<P, P::Mark>>
    where
        P: Process<'a, Self::Out>,
    {
        Seq {
            p: mp(self),
            q: mp(p),
        }

    }
    fn choice<PF, InF: 'a>(
        self,
        p: PF,
    ) -> PChoice<MarkedProcess<Self, Self::Mark>, MarkedProcess<PF, PF::Mark>>
    where
        PF: Process<'a, InF, Out = Self::Out>,
    {
        PChoice {
            pt: mp(self),
            pf: mp(p),
        }

    }
}

pub trait Graphfiller<'a> {
    fn fill_graph(self, g: &mut Graph<'a>) -> usize;
}

pub struct MarkedProcess<P, Mark: Im> {
    pub p: P,
    pd: PhantomData<Mark>,
}

impl<'a, P> Graphfiller<'a> for MarkedProcess<P, NotIm>
where
    P: Process<'a, (), Out = ()>,
{
    fn fill_graph(self, g: &mut Graph<'a>) -> usize {
        let (pni, pind, pno) = self.p.compile(g);
        g.set(pind, box pno);
        g.add(box pni)
    }
}

impl<'a, P> Graphfiller<'a> for MarkedProcess<P, IsIm>
where
    P: Process<'a, (), Out = ()>,
{
    fn fill_graph(self, g: &mut Graph<'a>) -> usize {
        let pnio = self.p.compileIm(g);
        g.add(box pnio)
    }
}




pub fn mp<'a, In: 'a, P>(p: P) -> MarkedProcess<P, P::Mark>
where
    P: Process<'a, In>,
{
    MarkedProcess {
        p: p,
        pd: PhantomData,
    }
}

//  _   _       _   _     _
// | \ | | ___ | |_| |__ (_)_ __   __ _
// |  \| |/ _ \| __| '_ \| | '_ \ / _` |
// | |\  | (_) | |_| | | | | | | | (_| |
// |_| \_|\___/ \__|_| |_|_|_| |_|\__, |
//                                |___/

pub struct PNothing{}

impl<'a> Process<'a, ()> for PNothing
{
    type Out = ();
    type NI = DummyN<()>;
    type NO = DummyN<()>;
    type NIO = Nothing;
    fn compileIm(self, _: &mut Graph) -> Self::NIO {
        Nothing{}
    }
    type Mark = IsIm;
}




//  _____      __  __       _
// |  ___| __ |  \/  |_   _| |_
// | |_ | '_ \| |\/| | | | | __|
// |  _|| | | | |  | | |_| | |_
// |_|  |_| |_|_|  |_|\__,_|\__|

impl<'a, F: 'a, In: 'a, Out: 'a> Process<'a, In> for F
where
    F: FnMut(In) -> Out,
{
    type Out = Out;
    type NI = DummyN<()>;
    type NO = DummyN<Out>;
    type NIO = FnMutN<F>;
    fn compileIm(self, _: &mut Graph) -> Self::NIO {
        FnMutN(self)
    }
    type Mark = IsIm;
}



//  ____
// / ___|  ___  __ _
// \___ \ / _ \/ _` |
//  ___) |  __/ (_| |
// |____/ \___|\__, |
//                |_|

// P and Q should be marked processes
pub struct Seq<P, Q> {
    p: P,
    q: Q,
}

impl<'a, P, Q, In: 'a, Mid: 'a, Out: 'a> Process<'a, In>
    for Seq<MarkedProcess<P, NotIm>, MarkedProcess<Q, NotIm>>
where
    P: Process<'a, In, Out = Mid>,
    Q: Process<'a, Mid, Out = Out>,
{
    type Out = Q::Out;
    type NI = P::NI;
    type NO = Q::NO;
    type NIO = DummyN<Out>;
    type Mark = NotIm;
    fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let (pni, pind, pno) = self.p.p.compile(g);
        let (qni, qind, qno) = self.q.p.compile(g);
        g.set(pind, box node!(pno >> qni));
        (pni, qind, qno)

    }
}

impl<'a, P, Q, In: 'a, Mid: 'a, Out: 'a> Process<'a, In>
    for Seq<MarkedProcess<P, IsIm>, MarkedProcess<Q, NotIm>>
where
    P: Process<'a, In, Out = Mid>,
    Q: Process<'a, Mid, Out = Out>,
{
    type Out = Q::Out;
    type NI = NSeq<P::NIO, Q::NI>;
    type NO = Q::NO;
    type NIO = DummyN<Out>;
    type Mark = NotIm;
    fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let pnio = self.p.p.compileIm(g);
        let (qni, qind, qno) = self.q.p.compile(g);
        (node!(pnio >> qni), qind, qno)

    }
}

impl<'a, P, Q, In: 'a, Mid: 'a, Out: 'a> Process<'a, In>
    for Seq<MarkedProcess<P, NotIm>, MarkedProcess<Q, IsIm>>
where
    P: Process<'a, In, Out = Mid>,
    Q: Process<'a, Mid, Out = Out>,
{
    type Out = Q::Out;
    type NI = P::NI;
    type NO = NSeq<P::NO, Q::NIO>;
    type NIO = DummyN<Out>;
    type Mark = NotIm;
    fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let (pni, pind, pno) = self.p.p.compile(g);
        let qnio = self.q.p.compileIm(g);
        (pni, pind, node!(pno >> qnio))

    }
}

impl<'a, P, Q, In: 'a, Mid: 'a, Out: 'a> Process<'a, In>
    for Seq<MarkedProcess<P, IsIm>, MarkedProcess<Q, IsIm>>
where
    P: Process<'a, In, Out = Mid>,
    Q: Process<'a, Mid, Out = Out>,
{
    type Out = Q::Out;
    type NI = DummyN<()>;
    type NO = DummyN<Out>;
    type NIO = NSeq<P::NIO, Q::NIO>;
    type Mark = IsIm;
    fn compileIm(self, g: &mut Graph<'a>) -> Self::NIO {
        let pnio = self.p.p.compileIm(g);
        let qnio = self.q.p.compileIm(g);
        node!(pnio >> qnio)
    }
}


//  ____
// |  _ \ __ _ _   _ ___  ___
// | |_) / _` | | | / __|/ _ \
// |  __/ (_| | |_| \__ \  __/
// |_|   \__,_|\__,_|___/\___|

#[derive(Copy, Clone, Debug)]
pub struct Pause {}

#[allow(non_upper_case_globals)]
pub static Pause: Pause = Pause {};

impl<'a, In: 'a> Process<'a, In> for Pause
where
    In: Default,
{
    type Out = In;
    type NI = NSeq<RcStore<In>, NPause>;
    type NO = RcLoad<In>;
    type NIO = DummyN<In>;
    type Mark = NotIm;
    fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let rcin = Rc::new(Cell::new(In::default()));
        let rcout = rcin.clone();
        let out = g.reserve();
        (node!(store(rcin) >> pause(out)), out, load(rcout))
    }
}

//   ____ _           _
//  / ___| |__   ___ (_) ___ ___
// | |   | '_ \ / _ \| |/ __/ _ \
// | |___| | | | (_) | | (_|  __/
//  \____|_| |_|\___/|_|\___\___|


pub struct PChoice<PT, PF> {
    pt: PT,
    pf: PF,
}
impl<'a, PT, PF, InT: 'a, InF: 'a, Out: 'a> Process<'a, ChoiceData<InT, InF>>
    for PChoice<MarkedProcess<PT, NotIm>, MarkedProcess<PF, NotIm>>
where
    PT: Process<'a, InT, Out = Out>,
    PF: Process<'a, InF, Out = Out>,
    Out: Default,
{
    type Out = Out;
    type NI = NChoice<PT::NI, PF::NI>;
    type NO = RcLoad<Out>;
    type NIO = DummyN<Out>;
    type Mark = NotIm;
    fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let (ptni, ptind, ptno) = self.pt.p.compile(g);
        let (pfni, pfind, pfno) = self.pf.p.compile(g);
        let rct = Rc::new(Cell::new(Out::default()));
        let rcf = rct.clone();
        let rcout = rct.clone();
        let out = g.reserve();
        g.set(ptind, box node!(ptno >> store(rct) >> jump(out)));
        g.set(pfind, box node!(pfno >> store(rcf) >> jump(out)));
        (node!(choice ptni pfni), out, load(rcout))

    }
}

impl<'a, PT, PF, InT: 'a, InF: 'a, Out: 'a> Process<'a, ChoiceData<InT, InF>>
    for PChoice<MarkedProcess<PT, IsIm>, MarkedProcess<PF, NotIm>>
where
    PT: Process<'a, InT, Out = Out>,
    PF: Process<'a, InF, Out = Out>,
    Out: Default,
{
    type Out = Out;
    type NI = NChoice<NSeq<PT::NIO, NSeq<RcStore<Out>, NJump>>, PF::NI>;
    type NO = RcLoad<Out>;
    type NIO = DummyN<Out>;
    type Mark = NotIm;
    fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let ptnio = self.pt.p.compileIm(g);
        let (pfni, pfind, pfno) = self.pf.p.compile(g);
        let rct = Rc::new(Cell::new(Out::default()));
        let rcf = rct.clone();
        let rcout = rct.clone();
        let out = g.reserve();
        g.set(pfind, box node!(pfno >> store(rcf) >> jump(out)));
        (
            node!(choice {ptnio >> store(rct)>>jump(out)} pfni),
            out,
            load(rcout),
        )

    }
}


impl<'a, PT, PF, InT: 'a, InF: 'a, Out: 'a> Process<'a, ChoiceData<InT, InF>>
    for PChoice<MarkedProcess<PT, NotIm>, MarkedProcess<PF, IsIm>>
where
    PT: Process<'a, InT, Out = Out>,
    PF: Process<'a, InF, Out = Out>,
    Out: Default,
{
    type Out = Out;
    type NI = NChoice<PT::NI, NSeq<PF::NIO, NSeq<RcStore<Out>, NJump>> >;
    type NO = RcLoad<Out>;
    type NIO = DummyN<Out>;
    type Mark = NotIm;
    fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let (ptni, ptind, ptno) = self.pt.p.compile(g);
        let pfnio = self.pf.p.compileIm(g);
        let rct = Rc::new(Cell::new(Out::default()));
        let rcf = rct.clone();
        let rcout = rct.clone();
        let out = g.reserve();
        g.set(ptind, box node!(ptno >> store(rct) >> jump(out)));
        (
            node!(choice ptni {pfnio >> store(rcf) >> jump(out)}),
            out,
            load(rcout),
        )

    }
}

impl<'a, PT, PF, InT: 'a, InF: 'a, Out: 'a> Process<'a, ChoiceData<InT, InF>>
    for PChoice<MarkedProcess<PT, IsIm>, MarkedProcess<PF, IsIm>>
    where
    PT: Process<'a, InT, Out = Out>,
    PF: Process<'a, InF, Out = Out>,
    Out: Default,
{
    type Out = Out;
    type NI = DummyN<()>;
    type NO = DummyN<Out>;
    type NIO = NChoice<PT::NIO,PF::NIO>;
    type Mark = IsIm;
    fn compileIm(self, g: &mut Graph<'a>) -> Self::NIO {
        let ptnio = self.pt.p.compileIm(g);
        let pfnio = self.pf.p.compileIm(g);
        node!(choice ptnio pfnio)
    }
}



// __        ___     _ _
// \ \      / / |__ (_) | ___
//  \ \ /\ / /| '_ \| | |/ _ \
//   \ V  V / | | | | | |  __/
//    \_/\_/  |_| |_|_|_|\___|

//struct While<P>(P);

// impl<'a, P, In: 'a, Out: 'a> Process<'a, In> for While<MarkedProcess<P, NotIm>>
// where
//     P: Process<
//         'a,
//         In,
//         Out = Out,
//     >,
//     In: Default,
// {
//     type Out = Q::Out;
//     type NI = P::NI;
//     type NO = Q::NO;
//     type NIO = DummyN<Out>;
//     type Mark = NotIm;
//     fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
//         let While(MarkedProcess { p: p, pd: _ }) = self;
//         let (pni, pind, pno) = p.compile(g);
//         // input one time to initialize the loop
//         let rcin = Rc::new(Cell::new(In::default()));
//         // beginning of the loop
//         let rcbeg = rcin.clone();
//         // end of the loop
//         let rcend = rcin.clone();
//         let first_node = RcLoad::new(rcbeg).nseq(pni);
//         let first_node_ind = g.add(box first_node);
//         (pni, qind, qno)
//     }
// }
