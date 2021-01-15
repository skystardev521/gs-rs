
use bindings::{
    ConnStatusType, PGconn, PQconnectStart, PQconnectPoll, PQconsumeInput,PQisBusy,
    PQerrorMessage, PQescapeByteaConn, PQescapeLiteral,PQescapeStringConn, PQfinish,PGresult,
    PQfreemem, PQgetResult,  PQsendQuery,PQsetnonblocking, PQstatus, PQunescapeBytea, PostgresPollingStatusType,
};

use std::ptr::null_mut;
use std::ffi::{CStr, CString};
use std::collections::VecDeque;
use std::os::raw::{c_char, c_uchar, c_void};

use op_base::State;
use op_base::IReqResp;
use pq_result::PqResult;

pub struct PqConn {
    /// 链接是否可用
    available: bool,
    /// 链接字符串
    conn_info: String,
    /// 转义字符串Buffer 
    buffer_vec: Vec<u8>,
    /// 转义字符串Buffer指针 
    buffer_i8ptr: *mut i8,

    /// PGconn链接指针 
    conn_ptr: *mut PGconn,
    /// sql命令请求队列
    queue: VecDeque<Box<dyn IReqResp>>,
}

impl Drop for PqConn {
    fn drop(&mut self) {
        self.close_conn();
    }
}

//use mini_utils;
#[inline(always)]
fn string2cstring(conn_info: String) -> CString {
    unsafe { CString::from_vec_unchecked(conn_info.into_bytes()) }
}
#[inline(always)]
fn charptr2string(c_char_ptr: *const c_char) -> String {
    unsafe { CStr::from_ptr(c_char_ptr).to_string_lossy().to_string() }
}

impl PqConn {
    /// conn_info:连接串; sql_buffer:min 16K
    pub fn new(conn_info: String, sql_buffer: usize) -> Result<Self, String> {
        let mut capacity = sql_buffer * 2;
        if capacity < 1024 * 32 {
            capacity = 1024 * 32;
        }
        
        let buffer_vec = vec![0u8; capacity];
        let buffer_i8ptr = buffer_vec.as_ptr() as *mut i8;

        let mut pq_conn = PqConn {
            conn_info,
            buffer_vec,
            buffer_i8ptr,
            available:false,
            conn_ptr: null_mut(),
            queue: VecDeque::new()
        };

        match pq_conn.connect_start(){
            Ok(())=>Ok(pq_conn),
            Err(err)=>Err(err)
        }
    }

    #[inline(always)]
    fn close_conn(&mut self){
        self.set_available(false);
        if !self.conn_ptr.is_null() {
            unsafe {
                PQfinish(self.conn_ptr);
                self.conn_ptr = null_mut();
            }
        }
    }

    #[inline(always)]
    fn connect_start(&mut self) -> Result<(), String> {
        let c_char_ptr = string2cstring(self.conn_info.clone());
        self.conn_ptr = unsafe { PQconnectStart(c_char_ptr.as_ptr()) };
        if !self.conn_ptr.is_null() {
            return Ok(());
        }
        return Err(self.get_err_msg());
    }

    /// 链接是否可用
    #[inline(always)]
    pub fn set_available(&mut self, val:bool){
        self.available = val;
    }

    #[inline(always)]
    pub fn connect_poll(&mut self) -> Result<bool, String> {
        let poll_status = unsafe { PQconnectPoll(self.conn_ptr) };

        if PostgresPollingStatusType::PGRES_POLLING_FAILED == poll_status {
            return Err(self.get_err_msg());
        }

        let conn_status = self.get_conn_status();

        if ConnStatusType::CONNECTION_OK == conn_status {
            self.set_available(true);
            unsafe {
                //0,阻塞    1,非阻塞
                PQsetnonblocking(self.conn_ptr, 1);
            }
            return Ok(true);
        }

        if ConnStatusType::CONNECTION_BAD == conn_status {
            return Err(self.get_err_msg());
        }
        return Ok(false);
    }

    /// 获取链接状态
    #[inline(always)]
    pub fn get_conn_status(&self) -> ConnStatusType {
        unsafe { PQstatus(self.conn_ptr) }
    }

    /// 获取错误信息
    #[inline(always)]
    pub fn get_err_msg(&self) -> String {
        unsafe { charptr2string(PQerrorMessage(self.conn_ptr)) }
    }

    /// 是否可以读取结果
    #[inline(always)]
    pub fn is_result_ready(&self) -> Result<bool, String> {
        if unsafe { PQconsumeInput(self.conn_ptr) } == 1 {
            return Ok(unsafe { PQisBusy(self.conn_ptr) == 0 });
        }
        // 是否要 self.close_conn()
        return Err(self.get_err_msg());
    }

    /// 二进制数据 转换 二进制数据的字符串 有问题
    fn escape_bytes(&self, bytes: &[u8]) -> Result<String, String> {
        let ret_len = null_mut();
        let ret_val =
            unsafe { PQescapeByteaConn(self.conn_ptr, bytes.as_ptr(), bytes.len(), ret_len) };
        if ret_val.is_null() || ret_len.is_null() {
            return Err(self.get_err_msg());
        }
        unsafe {
            let string = charptr2string(ret_val as *const i8);
            PQfreemem(ret_val as *mut c_void);
            return Ok(string);
        }
    }

    /// 二进制数据的字符串 转换 二进制数据 有问题
    pub fn unescape_bytes(binary: *mut c_uchar) -> Result<Vec<u8>, String> {
        let ret_len = null_mut();
        let ret_val = unsafe { PQunescapeBytea(binary, ret_len) };
        if ret_val.is_null() || ret_len.is_null() {
            return Err("malloc return null".into());
        }
        unsafe {
            let vec = Vec::from_raw_parts(ret_val, *ret_len, *ret_len);
            PQfreemem(ret_val as *mut c_void);
            return Ok(vec);
        }
    }

    /// 转义字符串 字符串 开始结束 增加 “'”
    pub fn escape_literal(&self, query: String) -> Result<String, String> {
        let cstr = string2cstring(query);
        let len = cstr.as_bytes().len();
        let c_char_ptr = unsafe { PQescapeLiteral(self.conn_ptr, cstr.as_ptr(), len) };
        if c_char_ptr.is_null() {
            return Err(self.get_err_msg());
        }
        let result = charptr2string(c_char_ptr);
        unsafe {
            PQfreemem(c_char_ptr as *mut c_void);
        }
        Ok(result)
    }

    /// 转义字符串 字符串 开始结束 不增加 “'” 这个函数比escape_literal性能高些
    pub fn escape_string(&mut self, query: String) -> Result<String, String> {
        let ret_err = null_mut();
        let cstr = string2cstring(query);
        let _ret_len = unsafe {
            PQescapeStringConn(
                self.conn_ptr,
                self.buffer_i8ptr,
                cstr.as_ptr(),
                cstr.as_bytes().len(),
                ret_err,
            )
        };
        if ret_err.is_null() {
            return Ok(charptr2string(self.buffer_i8ptr));
        } else {
            return Err(self.get_err_msg());
        }
    }
    
    /// PqResult:返回空指针之前 不能再次调用 send_query
    /// PQsendQuery:一次可以运行多个sql语句 有多条结果(PqResult)
    pub fn send_query(&mut self, query: String) -> Result<bool, String> {
        let cstr = string2cstring(query.clone());
        let ret = unsafe { PQsendQuery(self.conn_ptr, cstr.as_ptr()) };
        if ret == 1 {
            self.set_available(false);
            return Ok(true);
        }
        
        if self.get_conn_status() == ConnStatusType::CONNECTION_BAD{
            self.close_conn();
            match self.connect_start(){
                Ok(())=>{
                    return Ok(false);
                }
                Err(err)=>{
                    return Err(err);
                }
            }
        }
        return Err(self.get_err_msg());
    }

    pub fn get_result(&self) -> *mut PGresult {
        unsafe { PQgetResult(self.conn_ptr) }
    }
}

impl PqConn{

    pub fn get_queue_len(&self) -> usize {
        self.queue.len()
    }
    #[inline(always)]
    pub(crate) fn get_op(&mut self) -> Option<&mut Box<dyn IReqResp>> {
        self.queue.front_mut()
    }

    #[inline(always)]
    pub(crate) fn pop(&mut self) -> Option<Box<dyn IReqResp>> {
        self.queue.pop_front()
    }

    pub fn push(&mut self, op: Box<dyn IReqResp>) {
        if self.queue.len() == 1 {
            if let Some(mut_op) = self.queue.front_mut() {
                mut_op.set_state(State::Exec);
                mut_op.request();
            }
        }
    }
}