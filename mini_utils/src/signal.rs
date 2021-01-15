#[derive(Copy, Clone)]
struct Ctr {
    cur: usize,
}

#[allow(dead_code)]
impl Ctr {
    fn start() -> Self {
        Ctr { cur: 0 }
    }
    fn next(self) -> Self {
        Ctr { cur: self.cur + 1 }
    }
}

impl From<Ctr> for usize {
    fn from(ctr: Ctr) -> Self {
        ctr.cur
    }
}

trait Sig {
    type Data;
    type Receive: Rec;
    fn disc(&mut self, id: usize);
    fn emit(&self, data: Self::Data);
    fn conn(&mut self) -> Self::Receive;
}

trait Rec {
    type Data;
    fn get_id(&self) -> usize;
    fn on_emit(self, data: Self::Data);
}

#[macro_export]
macro_rules! def_signal {
    ($sig:ident,$rec:ident,$data:ty,$cls:expr) => {
        #[derive(Copy, Clone)]
        pub struct $rec {
            id: usize,
        }
        pub struct $sig {
            ctr: Ctr,
            recs: Vec<$rec>,
        }
        impl $rec {
            fn new(id: usize) -> Self {
                Self { id }
            }
        }
        impl $sig {
            fn new() -> Self {
                $sig {
                    recs: vec![],
                    ctr: Ctr::start(),
                }
            }
            fn next(&mut self) -> usize {
                self.ctr = self.ctr.next();
                self.ctr.into()
            }
        }
        impl Rec for $rec {
            type Data = $data;
            fn get_id(&self) -> usize {
                self.id
            }
            fn on_emit(self, data: Self::Data) {
                $cls(self, data);
            }
        }
        impl Sig for $sig {
            type Data = $data;
            type Receive = $rec;
            fn emit(&self, data: Self::Data) {
                self.recs.iter().for_each(|rec| rec.on_emit(data));
            }
            fn conn(&mut self) -> Self::Receive {
                let id = self.next();
                let rec = $rec::new(id);
                self.recs.push(rec);
                rec
            }
            fn disc(&mut self, id: usize) {
                if let Some(idx) = self.recs.iter().position(|&rec| rec.id == id) {
                    self.recs.remove(idx);
                }
            }
        }
    };
}

#[derive(Copy, Clone)]
struct MySigData {
    pub num: i32,
}

#[test]
fn test_signal() {
    //改动

    def_signal!(MySig, MyRec, MySigData, |rec: MyRec, data: MySigData| {
        println!("mySig receive R{} - num:{}", rec.id, data.num);
    });

    let mut ms2 = MySig::new();
    let r1 = ms2.conn();
    let r2 = ms2.conn();
    ms2.emit(MySigData { num: 3 });

    ms2.disc(r1.id);

    ms2.emit(MySigData { num: 9 });

    ms2.disc(r2.id);
}
