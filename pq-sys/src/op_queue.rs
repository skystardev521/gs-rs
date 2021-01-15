
use pq_conn::PqConn;
use op_base::IReqResp;
use op_base::OpBase;
use op_base::State;
use std::collections::VecDeque;

pub struct OpQueue {
    max_len: usize,
    conn_vec: Vec<PqConn>,
}

impl OpQueue {
    pub fn new(max_len: usize) -> Self {
        OpQueue {
            max_len,
            conn_vec: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn get_max_len(&self) -> usize {
        self.max_len
    }

    // 每个链接一个队列(通过hash分配)
    // 目的是同(hash)用同一个链接保证顺序
    pub fn push(&mut self, hash: u64, op: Box<dyn IReqResp>) {
        let len = self.conn_vec.len();
        if len == 0 {
            let mut op_mut = op;
            op_mut.set_state(State::Busy);
            op_mut.respone(None);
            return;
        }

        let idx = (hash % len as u64) as usize;
        let conn = &mut self.conn_vec[idx];
        if conn.get_queue_len() < self.max_len {
            conn.push(op);
        } else {
            let mut op_mut = op;
            op_mut.set_state(State::Busy);
            op_mut.respone(None);
        }
    }
}

#[test]
fn test() {
    let mut queue = OpQueue::new(9);
    //queue.push()
}
