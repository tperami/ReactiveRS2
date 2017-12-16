use node::*;
use super::*;


pub struct Par<P, Q> {
    pub(crate) p: P,
    pub(crate) q: Q,
}

impl<'a, P, Q, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a> Process<'a, (InP, InQ)>
    for Par<MarkedProcess<P, NotIm>, MarkedProcess<Q, NotIm>>
where
    P: Process<'a, InP, Out = OutP>,
    Q: Process<'a, InQ, Out = OutQ>,
{
    type Out = (OutP, OutQ);
    type NI = NSeq<NPar<P::NI, Q::NI>, Ignore>;
    type NO = NMerge<P::Out, Q::Out>;
    type NIO = DummyN<Self::Out>;
    type Mark = NotIm;
    fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let (pni, pind, pno) = self.p.p.compile(g);
        let (qni, qind, qno) = self.q.p.compile(g);
        let out_ind = g.reserve();
        let rc1 = new_rcjp();
        let rc2 = rc1.clone();
        let rcout = rc1.clone();
        g.set(pind, box node!(pno >> set1(rc1,out_ind)));
        g.set(qind, box node!(qno >> set2(rc2,out_ind)));
        (nodei!(pni || qni), out_ind, merge(rcout))
    }
}

impl<'a, P, Q, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a> Process<'a, (InP, InQ)>
    for Par<MarkedProcess<P, IsIm>, MarkedProcess<Q, NotIm>>
where
    P: Process<'a, InP, Out = OutP>,
    Q: Process<'a, InQ, Out = OutQ>,
{
    type Out = (OutP, OutQ);
    type NI = NSeq<NPar<NSeq<P::NIO, RcStore<OutP>>, Q::NI>, Ignore>;
    type NO = NSeq<GenP, NPar<RcLoad<OutP>, Q::NO>>;
    type NIO = DummyN<Self::Out>;
    type Mark = NotIm;
    fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let pnio = self.p.p.compileIm(g);
        let (qni, qind, qno) = self.q.p.compile(g);
        let rcin = new_rcell();
        let rcout = rcin.clone();
        (
            nodei!((pnio >> store(rcin)) || qni),
            qind,
            nodep!(load(rcout) || qno),
        )

    }
}

impl<'a, P, Q, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a> Process<'a, (InP, InQ)>
    for Par<MarkedProcess<P, NotIm>, MarkedProcess<Q, IsIm>>
where
    P: Process<'a, InP, Out = OutP>,
    Q: Process<'a, InQ, Out = OutQ>,
{
    type Out = (OutP, OutQ);
    type NI = NSeq<NPar<P::NI, NSeq<Q::NIO, RcStore<OutQ>>>, Ignore>;
    type NO = NSeq<GenP, NPar<P::NO, RcLoad<OutQ>>>;
    type NIO = DummyN<Self::Out>;
    type Mark = NotIm;
    fn compile(self, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let (pni, pind, pno) = self.p.p.compile(g);
        let qnio = self.q.p.compileIm(g);
        let rcin = new_rcell();
        let rcout = rcin.clone();
        (
            nodei!(pni || (qnio >> store(rcin))),
            pind,
            nodep!(pno || load(rcout)),
        )

    }
}

impl<'a, P, Q, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a> Process<'a, (InP, InQ)>
    for Par<MarkedProcess<P, IsIm>, MarkedProcess<Q, IsIm>>
where
    P: Process<'a, InP, Out = OutP>,
    Q: Process<'a, InQ, Out = OutQ>,
{
    type Out = (OutP, OutQ);
    type NI = DummyN<()>;
    type NO = DummyN<Self::Out>;
    type NIO = NPar<P::NIO, Q::NIO>;
    type Mark = IsIm;
    fn compileIm(self, g: &mut Graph<'a>) -> Self::NIO {
        let pnio = self.p.p.compileIm(g);
        let qnio = self.q.p.compileIm(g);
        node!(pnio || qnio)
    }
}