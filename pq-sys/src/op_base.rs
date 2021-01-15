use bindings::PGresult;

pub trait IOpBase {
    // 用于生成sql发送到Db
    fn set_state(&mut self, state: State);
}

// send,recv db
pub trait IReqResp: IOpBase {
    // 生成Sql发送Db
    fn request(&self);
    // 读Db反回的来结果
    fn respone(&self, result: Option<PGresult>);
}

#[derive(Debug, Copy, Clone)]
pub enum State {
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

pub struct OpBase {
    // 用于分配Db链接
    // 列如使用用户Id
    hash: u64,
    // 时间戳用于超时
    time: u64,
    // 当前运行状态
    state: State,
}

impl OpBase {
    pub fn new(hash: u64) -> Self {
        OpBase {
            hash,
            time: 0,
            state: State::Succ,
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
    pub fn get_state(&self) -> State {
        self.state
    }
}

#[macro_export]
macro_rules! MakeOpBase {
    ($name:ident, $field:ident) => {
        impl $crate::async::IOpBase for $name {
            fn set_state(&mut self, state: $crate::async::State) {
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
