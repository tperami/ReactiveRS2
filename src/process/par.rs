use node::*;
use super::*;


pub struct Par<P, Q>(pub(crate) P, pub(crate) Q);

impl<'a, P, Q, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a> IntProcess<'a, (InP, InQ)>
    for Par<P,Q>
    where
    P: Process<'a, InP, Out = OutP>,
    Q: Process<'a, InQ, Out = OutQ>,
{
    type Out = (OutP,OutQ);
    fn printDot(&mut self,curNum : &mut usize) -> (usize,usize){
        let (begp,endp) = self.0.printDot(curNum);
        let (begq,endq) = self.1.printDot(curNum);
        let numbeg = *curNum;
        let numend = numbeg +1;
        *curNum += 2;
        println!("{} [shape = triangle, label = \"\"]",numbeg);
        println!("{}:sw -> {}:n [label = \"{}\"]",numbeg,begp,tname::<InP>());
        println!("{}:se -> {}:n [label = \"{}\"]",numbeg,begq,tname::<InQ>());
        println!("{} [shape= invtriangle, label = \"\"]",numend);
        println!("{}:s -> {}:nw [label = \"{}\"]",endp,numend,tname::<OutP>());
        println!("{}:s -> {}:ne [label = \"{}\"]",endq,numend,tname::<OutQ>());
        (numbeg,numend)
    }
}

// NI - NI
implNI!{
    (InP,InQ),
    impl<'a, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a, PNI, PNO, QNI, QNO>
        for Par<ProcessNotIm<'a, InP, OutP, PNI, PNO>, ProcessNotIm<'a, InQ, OutQ, QNI, QNO>>
        where
        PNI: Node<'a, InP, Out = ()>,
        PNO: Node<'a, (), Out = OutP>,
        QNI: Node<'a, InQ, Out = ()>,
        QNO: Node<'a, (), Out = OutQ>,

    trait IntProcessNotIm<'a, (InP,InQ)>
    {
        type NI = NSeq<NPar<PNI, QNI>, Ignore>;
        type NO = NMerge<OutP, OutQ>;
        fn compile(self: Box<Self>, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
            let s = *self;
            let Par(p, q) = s;
            let (pni, pind, pno) = p.compile(g);
            let (qni, qind, qno) = q.compile(g);
            let out_ind = g.reserve();
            let rc1 = new_rcjp();
            let rc2 = rc1.clone();
            let rcout = rc1.clone();
            g.set(pind, box node!(pno >> set1(rc1, out_ind)));
            g.set(qind, box node!(qno >> set2(rc2, out_ind)));
            (nodei!(pni || qni), out_ind, merge(rcout))
        }

    }
}

// Im - NI
implNI!{
    (InP,InQ),
    impl<'a, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a, PNIO, QNI, QNO>
        for Par<ProcessIm<'a, InP, OutP, PNIO>, ProcessNotIm<'a, InQ, OutQ, QNI, QNO>>
        where
        PNIO: Node<'a, InP, Out = OutP>,
        QNI: Node<'a, InQ, Out = ()>,
        QNO: Node<'a, (), Out = OutQ>,

    trait IntProcessNotIm<'a, (InP,InQ)>
    {
        type NI = NSeq<NPar<NSeq<PNIO, RcStore<OutP>>, QNI>, Ignore>;
        type NO = NSeq<GenP, NPar<RcLoad<OutP>, QNO>>;
        fn compile(self: Box<Self>, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
            let s = *self;
            let Par(p, q) = s;
            let pnio = p.compileIm(g);
            let (qni, qind, qno) = q.compile(g);
            let rcin = new_rcell();
            let rcout = rcin.clone();
            (
                nodei!((pnio >> store(rcin)) || qni),
                qind,
                nodep!(load(rcout) || qno),
            )

        }

    }
}

// NI - Im
implNI!{
    (InP,InQ),
    impl<'a, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a, PNI, PNO, QNIO>
        for Par<ProcessNotIm<'a, InP, OutP, PNI, PNO>, ProcessIm<'a, InQ, OutQ, QNIO>>
        where
        PNI: Node<'a, InP, Out = ()>,
        PNO: Node<'a, (), Out = OutP>,
        QNIO: Node<'a, InQ, Out = OutQ>,

    trait IntProcessNotIm<'a, (InP,InQ)>
    {
        type NI = NSeq<NPar<PNI, NSeq<QNIO, RcStore<OutQ>>>, Ignore>;
        type NO = NSeq<GenP, NPar<PNO, RcLoad<OutQ>>>;
        fn compile(self: Box<Self>, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
            let s = *self;
            let Par(p, q) = s;
            let (pni, pind, pno) = p.compile(g);
            let qnio = q.compileIm(g);
            let rcin = new_rcell();
            let rcout = rcin.clone();
            (
                nodei!(pni || (qnio >> store(rcin))),
                pind,
                nodep!(pno || load(rcout)),
            )

        }

    }
}

// Im - Im
implIm!{
    (InP,InQ),
    impl<'a, InP: 'a, InQ: 'a, OutP: 'a, OutQ: 'a, PNIO, QNIO>
        for Par<ProcessIm<'a, InP, OutP, PNIO>, ProcessIm<'a, InQ, OutQ, QNIO>>
        where
        PNIO: Node<'a, InP, Out = OutP>,
        QNIO: Node<'a, InQ, Out = OutQ>,

    trait IntProcessIm<'a, (InP,InQ)>
    {
        type NIO = NPar<PNIO, QNIO>;
        fn compileIm(self : Box<Self>, g: &mut Graph<'a>) -> Self::NIO {
            let s = *self;
            let Par(p, q) = s;
            let pnio = p.compileIm(g);
            let qnio = q.compileIm(g);
            node!(pnio || qnio)
        }
    }
}





//  ____  _
// | __ )(_) __ _
// |  _ \| |/ _` |
// | |_) | | (_| |
// |____/|_|\__, |
//          |___/


pub struct BigPar<P>(pub(crate) Vec<P>);

impl<'a, P, In: 'a> IntProcess<'a, In> for BigPar<P>
where
    P: Process<'a, In, Out = ()>,
    In: Copy,
{
    type Out = ();
    fn printDot(&mut self, curNum: &mut usize) -> (usize, usize) {
        let num = *curNum;
        *curNum += 1;
        println!("{} [shape = box, label= \"BigPar\"];", num);
        (num, num)
    }
}

impl<'a, In: 'a, PNI, PNO> IntProcessNotIm<'a, In> for BigPar<ProcessNotIm<'a,In,(),PNI,PNO>>
where
    PNI: Node<'a, In, Out = ()>,
    PNO: Node<'a, (), Out = ()>,
    In: Copy,
{
    type NI = NSeq<RcStore<In>, NBigPar>;
    type NO = Nothing;
    fn compile(self: Box<Self>, g: &mut Graph<'a>) -> (Self::NI, usize, Self::NO) {
        let mut dests: Vec<usize> = vec![];
        let end_point = g.reserve();
        let rcbjp = new_rcbjp(self.0.len(), end_point);
        let rcin = new_rcell();
        for p in self.0 {
            let (pni, pind, pno) = p.compile(g);
            g.set(pind, box node!(pno >> big_merge(rcbjp.clone())));
            dests.push(g.add(box node!(load_copy(rcin.clone()) >> pni)));
        }
        (node!(store(rcin) >> NBigPar { dests }), end_point, Nothing)
    }
}