//use mini_utils::misc::MakeDeref;
//use mini_socket::os_epoll::OsEpoll;

use std::collections::VecDeque;

// 跨线程使用enum通信
// 把数据库果发送过来
pub enum DbRespData {
    AddUserData(Box<(ReqData, dyn FnOnce(Result<OpState, ReqData>))>),
    GetUserData(
        Box<(
            Result<OpState, (ReqData, RespData)>,
            dyn FnOnce(Result<OpState, (ReqData, RespData)>),
        )>,
    ),
}

#[derive(Debug, Copy, Clone)]
pub enum OpState {
    // 等待执行
    Wait,
    // 等待超时
    Busy,
    // 正在执行
    Exec,
    // 执行成功
    Succ,
    // 执行失败
    Fail,
}

pub struct Op {
    // 用于分配Db链接
    // 列如使用用户Id
    hash_id: u64,
    // 时间戳用于超时
    op_time: u64,
    // 当前运行状态
    state: OpState,
}

pub trait TOp {
    // 用于生成sql发送到Db
    fn set_state(&mut self, state: OpState);
}

pub trait TOpReq: TOp {
    // 用于生成sql发送到Db
    fn request(&mut self);
    //fn set_state(&mut self, state:OpState);
}

pub trait TOpResp: TOpReq {
    // PGresult
    //fn respone(&mut self, PGresult);
    // 响应Db反回的数据结果
    fn respone(&mut self);
}

impl Op {
    pub fn new(hash_id: u64) -> Self {
        Op {
            hash_id,
            op_time: 0,
            state: OpState::Succ,
        }
    }

    pub fn get_hash_id(&self) -> u64 {
        self.hash_id
    }

    pub fn get_op_time(&self) -> u64 {
        self.op_time
    }

    pub fn get_state(&self) -> OpState {
        self.state
    }
}

pub struct OpMgmt {
    max_len: usize,
    queue: VecDeque<Box<dyn TOpResp>>,
}

impl OpMgmt {
    pub fn new(max_len: usize) -> Self {
        OpMgmt {
            max_len,
            queue: VecDeque::new(),
        }
    }

    pub fn get_len(&self) -> usize {
        self.queue.len()
    }

    pub fn pop(&mut self) -> Option<Box<dyn TOpResp>> {
        self.queue.pop_front()
    }

    pub fn push(&mut self, op: Box<dyn TOpResp>) {
        if self.queue.len() < self.max_len {
            self.queue.push_back(op);
        } else {
            let mut op_mut = op;
            op_mut.set_state(OpState::Busy);
            op_mut.respone();
        }
    }
}
/*
#[macro_export]
macro_rules! MakeBaseOp {
    ($name:ident, $field:ident) => {
        impl TOp for $name {
            fn set_state(&mut self, state: OpState) {
                self.state = state;
            }
        }

        impl std::ops::Deref for $name {
            type Target = Op; //$target;
            fn deref<'a>(&'a self) -> &'a Self::Target {
                &self.$field
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target {
                &mut self.$field
            }
        }
    };
}
*/
#[derive(Debug)]
pub struct ReqData {
    id: u64,
}

#[derive(Debug)]
pub struct RespData {
    name: String,
}

pub struct QueryUser {
    op: Op,
    // 请求的数据用生生成sql
    req_data: ReqData,
    // 执行sql后反回的结果数据
    resp_data: Option<RespData>,
}

/*
MakeBaseOp!(QueryUser, op);

impl TOpReq for QueryUser {
    fn request(&mut self) {
        println!(
            "req_data:{:?}, resp_data:{:?} line:{}",
            self.req_data,
            self.resp_data,
            line!()
        );
    }
}
*/
/*
impl TOpResp for QueryUser {
    fn respone(&mut self) {

        println!(
            "req_data:{:?}, resp_data:{:?} line:{}",
            self.req_data,
            self.resp_data,
            line!()
        );
        if let Some(xx) = &self.resp_data {}
    }
}
*/

#[test]
fn test() {
    let mut db_op = QueryUser {
        op: Op::new(1),
        req_data: ReqData { id: 1 },
        resp_data: Some(RespData {
            name: "name".into(),
        }),
    };
}
