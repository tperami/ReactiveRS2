use node::*;
use super::*;


impl<'a, P, Q, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a> ProcessPar<'a, (InP, InQ)>
    for Par<MarkedProcessPar<P, NotIm>, MarkedProcessPar<Q, NotIm>>
where
    P: ProcessPar<'a, InP, Out = OutP>,
    Q: ProcessPar<'a, InQ, Out = OutQ>,
    OutP: Send + Sync,
    OutQ: Send + Sync,
{
    type NI = NSeq<NPar<P::NI, Q::NI>, Ignore>;
    type NO = NMergePar<P::Out, Q::Out>;
    type NIO = DummyN<Self::Out>;
    type Mark = NotIm;
    type MarkOnce = And<P::MarkOnce, Q::MarkOnce>;
    fn compile_par(self, g: &mut GraphPar<'a>) -> (Self::NI, usize, Self::NO) {
        let (pni, pind, pno) = self.p.p.compile_par(g);
        let (qni, qind, qno) = self.q.p.compile_par(g);
        let out_ind = g.reserve();
        let rc1 = new_arcjp();
        let rc2 = rc1.clone();
        let rcout = rc1.clone();
        g.set(pind, box node!(pno >> set1_par(rc1, out_ind)));
        g.set(qind, box node!(qno >> set2_par(rc2, out_ind)));
        (nodei!(pni || qni), out_ind, merge_par(rcout))
    }
}

impl<'a, P, Q, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a> ProcessPar<'a, (InP, InQ)>
    for Par<MarkedProcessPar<P, IsIm>, MarkedProcessPar<Q, NotIm>>
where
    P: ProcessPar<'a, InP, Out = OutP>,
    Q: ProcessPar<'a, InQ, Out = OutQ>,
    OutP: Send + Sync,
    OutQ: Send + Sync,
{
    type NI = NSeq<NPar<NSeq<P::NIO, ArcStore<OutP>>, Q::NI>, Ignore>;
    type NO = NSeq<GenP, NPar<ArcLoad<OutP>, Q::NO>>;
    type NIO = DummyN<Self::Out>;
    type Mark = NotIm;
    type MarkOnce = And<P::MarkOnce, Q::MarkOnce>;
    fn compile_par(self, g: &mut GraphPar<'a>) -> (Self::NI, usize, Self::NO) {
        let pnio = self.p.p.compileIm_par(g);
        let (qni, qind, qno) = self.q.p.compile_par(g);
        let rcin = new_amutex();
        let rcout = rcin.clone();
        (
            nodei!((pnio >> store_par(rcin)) || qni),
            qind,
            nodep!(load_par(rcout) || qno),
        )

    }
}

impl<'a, P, Q, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a> ProcessPar<'a, (InP, InQ)>
    for Par<MarkedProcessPar<P, NotIm>, MarkedProcessPar<Q, IsIm>>
where
    P: ProcessPar<'a, InP, Out = OutP>,
    Q: ProcessPar<'a, InQ, Out = OutQ>,
    OutP: Send + Sync,
    OutQ: Send + Sync,
{
    type NI = NSeq<NPar<P::NI, NSeq<Q::NIO, ArcStore<OutQ>>>, Ignore>;
    type NO = NSeq<GenP, NPar<P::NO, ArcLoad<OutQ>>>;
    type NIO = DummyN<Self::Out>;
    type Mark = NotIm;
    type MarkOnce = And<P::MarkOnce, Q::MarkOnce>;
    fn compile_par(self, g: &mut GraphPar<'a>) -> (Self::NI, usize, Self::NO) {
        let (pni, pind, pno) = self.p.p.compile_par(g);
        let qnio = self.q.p.compileIm_par(g);
        let rcin = new_amutex();
        let rcout = rcin.clone();
        (
            nodei!(pni || (qnio >> store_par(rcin))),
            pind,
            nodep!(pno || load_par(rcout)),
        )

    }
}

impl<'a, P, Q, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a> ProcessPar<'a, (InP, InQ)>
    for Par<MarkedProcessPar<P, IsIm>, MarkedProcessPar<Q, IsIm>>
where
    P: ProcessPar<'a, InP, Out = OutP>,
    Q: ProcessPar<'a, InQ, Out = OutQ>,
    OutP: Send + Sync,
    OutQ: Send + Sync,
{
    type NI = DummyN<()>;
    type NO = DummyN<Self::Out>;
    type NIO = NPar<P::NIO, Q::NIO>;
    type Mark = IsIm;
    type MarkOnce = And<P::MarkOnce, Q::MarkOnce>;
    fn compileIm_par(self, g: &mut GraphPar<'a>) -> Self::NIO {
        let pnio = self.p.p.compileIm_par(g);
        let qnio = self.q.p.compileIm_par(g);
        node!(pnio || qnio)
    }
}

//  ____  _
// | __ )(_) __ _
// |  _ \| |/ _` |
// | |_) | | (_| |
// |____/|_|\__, |
//          |___/


impl<'a, P, In: 'a> ProcessPar<'a, In> for BigPar<MarkedProcessPar<P, NotIm>>
where
    P: ProcessPar<'a, In, Out = ()>,
    In: Copy + Send + Sync,
{
    type NI = NSeq<ArcStore<In>,NBigPar>;
    type NO = Nothing;
    type NIO = DummyN<Self::Out>;
    type Mark = NotIm;
    type MarkOnce = P::MarkOnce;
    fn compile_par(self, g: &mut GraphPar<'a>) -> (Self::NI, usize, Self::NO) {
        let mut dests: Vec<usize> = vec![];
        let end_point = g.reserve();
        let arcbjp = new_arcbjp(self.vp.len(),end_point);
        let arcin = new_amutex();
        for p in self.vp{
            let (pni, pind, pno) = p.p.compile_par(g);
            g.set(pind, box node!(pno >> big_merge_par(arcbjp.clone())));
            dests.push(g.add(box node!(load_copy_par(arcin.clone()) >> pni)));
        };
        (node!(store_par(arcin) >> NBigPar{dests}),end_point,Nothing)
    }
}