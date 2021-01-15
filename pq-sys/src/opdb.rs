use bindings::{
    ConnStatusType, PGconn, PGresult, PQclear, PQcmdTuples, PQconnectdb, PQconsumeInput,
    PQerrorMessage, PQescapeByteaConn, PQescapeStringConn, PQfinish, PQfname, PQftype, PQgetResult,
    PQgetisnull, PQgetlength, PQgetvalue, PQisBusy, PQresultStatus, PQsendQuery, PQsetnonblocking,
    PQstatus, PQunescapeBytea, CONNECTION_OK,
};

// send,recv db
pub trait Topdb {
    // 生成Sql发送Db
    fn send(&self);
    // 读Db反回的来结果
    fn recv(&self, result: PGresult);
}

pub trait TOp {
    // 用于生成sql发送到Db
    fn set_state(&mut self, state: OpState);
}

pub trait TReqResp: TOp {
    // Db响应回调函数
    fn resp_cb(&self);
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
    hash: u64,
    // 时间戳用于超时
    time: u64,
    // 当前运行状态
    state: OpState,
}

impl Op {
    pub fn new(hash: u64) -> Self {
        Op {
            hash,
            time: 0,
            state: OpState::Succ,
        }
    }
    #[inline(always)]
    pub fn get_hash(&self) -> u64 {
        self.hash
    }
    #[inline(always)]
    pub fn get_time(&self) -> u64 {
        self.time
    }

    #[inline(always)]
    pub fn get_state(&self) -> OpState {
        self.state
    }
}

#[macro_export]
macro_rules! MakeBaseOp {
    ($name:ident, $field:ident) => {
        impl $crate::opdb::TOp for $name {
            fn set_state(&mut self, state: $crate::opdb::OpState) {
                self.state = state;
            }
        }

        impl std::ops::Deref for $name {
            type Target = $crate::opdb::Op; //$target;
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

mod ReqResp {

    use super::opdbMod::opdbUserData;
    use super::Op;
    use super::TReqResp;
    use super::Topdb;

    // 用于发送给Db的数据样板
    pub struct GetUseData {
        op: Op,
        // 私有的
        id: u64,
        // 私有的
        data: Vec<u32>,
    }

    /*
    impl GetUseData {
        // 只能这种方法读数据
        pub fn get_id(&self) -> u64 {
            self.id
        }
        // 只能这种方法读数据
        pub fn set_data(&mut self, data: Vec<u32>) {
            self.data = data;
        }
    }
    */

    MakeBaseOp!(GetUseData, op);

    impl TReqResp for GetUseData {
        fn resp_cb(&self) {
            println!("GetUseData id:{}", self.id);
        }
    }

    #[test]
    fn test() {
        let mut opdb_obj_vec: Vec<Box<dyn Topdb>> = vec![];

        let mut req_resp = GetUseData {
            id: 1,
            data: vec![],
            op: Op::new(1),
        };

        req_resp.id = 10;

        let opdb_obj = opdbUserData::new(req_resp);

        opdb_obj_vec.push(Box::new(opdb_obj));
    }
}

mod opdbMod {

    use super::ReqResp::GetUseData;
    use super::Topdb;
    use bindings::PGresult;

    pub struct opdbUserData {
        req_resp: GetUseData,
    }

    impl Topdb for opdbUserData {
        fn send(&self) {
            //self.req_resp.id = 10;
        }
        fn recv(&self, res: PGresult) {}
    }

    impl opdbUserData {
        pub fn new(req_resp: GetUseData) -> Self {
            opdbUserData { req_resp }
        }
    }
}
