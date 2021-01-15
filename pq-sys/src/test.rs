#![allow(dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals)]

/*
use bindings::{
    ConnStatusType, ExecStatusType, PGconn, PGresult, PQclear, PQcmdTuples, PQconnectPoll,
    PQconnectStart, PQconsumeInput, PQerrorMessage, PQescapeByteaConn, PQescapeLiteral,
    PQescapeStringConn, PQfinish, PQfname, PQfnumber, PQfreemem, PQftype, PQgetResult, PQgetisnull,
    PQgetlength, PQgetvalue, PQisBusy, PQnfields, PQntuples, PQresultStatus, PQsendQuery,
    PQsetnonblocking, PQstatus, PQunescapeBytea, PostgresPollingStatusType,
};


use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uchar, c_void};
use std::time::Duration;
use std::str::FromStr;
use std::ptr::null_mut;

#[macro_use]
use bytes::{self,Bytes};

/// len:-1(不定长度)
enum PqType {
    /// len:1
    bool = 16,
    /// len:-1
    bytea = 17,
    /// len:1
    char = 18,
    /// len:63
    name = 19,
    /// len:8
    int8 = 20,
    /// len:2
    int2 = 21,
    /// len:-1
    int2vector = 22,
    /// len:4
    int4 = 23,
    /// len:-1
    text = 25,
    /// len:4
    oid = 26,
    /// len:-1
    oidvector = 30,
    /// len:-1
    json = 114,
    /// len:-1
    xml = 142,
    /// len:4
    float4 = 700,
    /// len:8
    float8 = 701,
    /// len:8
    money = 790,
    /// len:-1
    /// fixed length
    bpchar = 1042,
    /// len:-1
    /// variable length
    varchar = 1043,
    /// len:4
    date = 1082,
    /// len:8
    time = 1083,
    /// len:8
    timestamp = 1114,
    /// len:8
    timestamptz = 1184,
    /// len:16
    interval = 1186,
    /// len:12
    timetz = 1266,
    /// len:-1
    bit = 1560,
    /// len:-1
    varbit = 1562,
    /// len:-1
    numeric = 1700,
    /// len:16
    uuid = 2950,
    /// len:-1
    jsonb = 3802,
    /// len:-1
    jsonpath = 4072,
    /// len:-1
    int4range = 3904,
    /// len:-1
    numrange = 3906,
    /// len:-1
    daterange = 3912,
    /// len:-1
    int8range = 3926,
    /// len:-1
    cstring = 2275,
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

struct PqResult {
    ptr: *mut PGresult,
}
pub struct PqConn {
    conn_info: String,
    buffer_vec: Vec<u8>,
    buffer_i8ptr: *mut i8,
    conn_ptr: *mut PGconn,
    //queue: VecDeque<Box<dyn IReqResp>>,
}
impl Drop for PqResult {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                PQclear(self.ptr);
            }
        }
    }
}
impl Drop for PqConn {
    fn drop(&mut self) {
        self.close_conn();
    }
}

impl PqConn {
    /// conn_info:连接串; buffer_size:min 16K
    pub fn new(conn_info: String, buffer_size: usize) -> Self {
        let mut capacity = buffer_size * 2;
        if capacity < 1024 * 32 {
            capacity = 1024 * 32;
        }
        
        let buffer_vec = vec![0u8; capacity];
        let buffer_i8ptr = buffer_vec.as_ptr() as *mut i8;

        PqConn {
            conn_info,
            buffer_vec,
            buffer_i8ptr,
            conn_ptr: null_mut(),
        }
    }

    #[inline(always)]
    fn close_conn(&mut self){
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

    #[inline(always)]
    fn connect_poll(&self) -> Result<bool, String> {
        let poll_status = unsafe { PQconnectPoll(self.conn_ptr) };

        if PostgresPollingStatusType::PGRES_POLLING_FAILED == poll_status {
            return Err(self.get_err_msg());
        }

        let conn_status = unsafe { PQstatus(self.conn_ptr) };

        if ConnStatusType::CONNECTION_OK == conn_status {
            //0,阻塞    1,非阻塞
            unsafe {
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
    fn is_result_ready(&self) -> Result<bool, String> {
        if unsafe { PQconsumeInput(self.conn_ptr) } == 1 {
            return Ok(unsafe { PQisBusy(self.conn_ptr) == 0 });
        }
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

    fn get_result(&self) -> PqResult {
        PqResult::new(unsafe { PQgetResult(self.conn_ptr) })
    }
}

impl PqResult {
    pub fn new(ptr: *mut PGresult) -> Self {
        PqResult { ptr }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    #[inline(always)]
    pub fn get_rows(&self) -> i32 {
        unsafe { PQntuples(self.ptr) }
    }
    #[inline(always)]
    pub fn get_clos(&self) -> i32 {
        unsafe { PQnfields(self.ptr) }
    }

    #[inline(always)]
    pub fn get_status(&self) -> ExecStatusType {
        unsafe { PQresultStatus(self.ptr) }
    }

    #[inline(always)]
    pub fn get_type(&self, col: i32) -> u32 {
        unsafe { PQftype(self.ptr, col) }
    }

    /// 根据列名获取 数据所在的列
    #[inline(always)]
    pub fn get_col(&self, fname: &str) -> i32 {
        unsafe { PQfnumber(self.ptr, fname.as_ptr() as *const i8) }
    }

    /// 反回 当前 行,列 的数据长度
    #[inline(always)]
    pub fn get_val_len(&self, row: i32, col: i32) -> i32 {
        unsafe { PQgetlength(self.ptr, row, col) }
    }

    #[inline(always)]
    pub fn get_fname(&self, col: i32) -> *mut ::std::os::raw::c_char {
        unsafe { PQfname(self.ptr, col) }
    }
    /// 反回 当前 行,列 的数据长度
    #[inline(always)]
    pub fn get_val(&self, row: i32, col: i32) -> *mut ::std::os::raw::c_char {
        unsafe { PQgetvalue(self.ptr, row, col) }
    }

    /// false:non null true:null
    #[inline(always)]
    pub fn is_null_val(&self, row: i32, col: i32) -> bool {
        unsafe { PQgetisnull(self.ptr, row, col) == 1 }
    }

    /// 返回被 SQL 命令影响的行的数量
    #[inline(always)]
    pub fn get_cmd_rows(&self) -> *mut ::std::os::raw::c_char {
        unsafe { PQcmdTuples(self.ptr) }
    }

    pub fn get_i8_val(&self, row: i32, col: i32)->i8{
        unsafe{*self.get_val(row, col)}
    }

    pub fn get_u8_val(&self, row: i32, col: i32)->u8{
        self.get_val(row, col) as u8
    }

    pub fn get_i16_val(&self, row: i32, col: i32)->i16{
        let val = self.get_val(row, col);
        let strval = unsafe{CStr::from_ptr(val).to_string_lossy()};
        match i16::from_str(&strval){
            Ok(val)=>val,
            Err(err)=>{
                println!("get_i16_val({},{}) err:{}", row, col, err);
                return 0;
            }
        }
    }

    pub fn get_u16_val(&self, row: i32, col: i32)->u16{
        let val = self.get_val(row, col);
        let strval = unsafe{CStr::from_ptr(val).to_string_lossy()};
        match u16::from_str(&strval){
            Ok(val)=>val,
            Err(err)=>{
                println!("get_u16_val({},{}) err:{}", row, col, err);
                return 0;
            }
        }
    }

    pub fn get_i32_val(&self, row: i32, col: i32)->i32{
        let val = self.get_val(row, col);
        let strval = unsafe{CStr::from_ptr(val).to_string_lossy()};
        match i32::from_str(&strval){
            Ok(val)=>val,
            Err(err)=>{
                println!("get_i32_val({},{}) err:{}", row, col, err);
                return 0;
            }
        }
    }

    pub fn get_u32_val(&self, row: i32, col: i32)->u32{
        let val = self.get_val(row, col);
        let strval = unsafe{CStr::from_ptr(val).to_string_lossy()};
        match u32::from_str(&strval){
            Ok(val)=>val,
            Err(err)=>{
                println!("get_u32_val({},{}) err:{}", row, col, err);
                return 0;
            }
        }
    }

    pub fn get_i64_val(&self, row: i32, col: i32)->i64{
        let val = self.get_val(row, col);
        let strval = unsafe{CStr::from_ptr(val).to_string_lossy()};
        match i64::from_str(&strval){
            Ok(val)=>val,
            Err(err)=>{
                println!("get_i64_val({},{}) err:{}", row, col, err);
                return 0;
            }
        }
    }

    pub fn get_u64_val(&self, row: i32, col: i32)->u64{
        let val = self.get_val(row, col);
        let strval = unsafe{CStr::from_ptr(val).to_string_lossy()};
        match u64::from_str(&strval){
            Ok(val)=>val,
            Err(err)=>{
                println!("get_u64_val({},{}) err:{}", row, col, err);
                return 0;
            }
        }
    }

    pub fn get_f32_val(&self, row: i32, col: i32)->f32{
        let val = self.get_val(row, col);
        let strval = unsafe{CStr::from_ptr(val).to_string_lossy()};
        match f32::from_str(&strval){
            Ok(val)=>val,
            Err(err)=>{
                println!("get_i64_val({},{}) err:{}", row, col, err);
                return 0.0;
            }
        }
    }

    pub fn get_f64_val(&self, row: i32, col: i32)->f64{
        let val = self.get_val(row, col);
        let strval = unsafe{CStr::from_ptr(val).to_string_lossy()};
        match f64::from_str(&strval){
            Ok(val)=>val,
            Err(err)=>{
                println!("get_u64_val({},{}) err:{}", row, col, err);
                return 0.0;
            }
        }
    }

    pub fn get_string_val(&self, row: i32, col: i32)->String{
        let val = self.get_val(row, col);
        unsafe{CStr::from_ptr(val)}.to_string_lossy().to_string()
    }
}
*/

use std::time::Duration;
use pq_conn::PqConn;
use pq_result::PqResult;
use pq_result::{PqType};


#[test]
fn test() {
    
    let conninfo = String::from(
        "host=172.19.162.49 port=5432 dbname=postgres
        user=postgres password=postgres connect_timeout=10",
    );

    let mut pq_conn = match PqConn::new(conninfo, 1024){
        Ok(conn)=>conn,
        Err(err)=>{
            println!("connect_start error:{}", err);
            return;
        }
    };

    loop {
        std::thread::sleep(Duration::from_millis(100));
        match pq_conn.connect_poll() {
            Ok(true) => {
                println!("connect success");
                break;
            }
            Ok(false) => println!("connecting ..."),
            Err(err) => {
                println!("connect err:{}", err);
                return;
            }
        }
    }

    println!(
        "escape_literal:{}",
        pq_conn.escape_literal("".into()).unwrap()
    );
    println!("escape_string:{}", pq_conn.escape_string("".into()).unwrap());

    let query = String::from("que'ry");
    println!(
        "escape_literal:{}",
        pq_conn.escape_literal(query.clone()).unwrap()
    );
    
    println!("escape_string:{}", pq_conn.escape_string(query.clone()).unwrap());

    println!("escape_string:{}", pq_conn.escape_string(query.clone()).unwrap());
    

    //"select * from test where name = 'name'".into()
    match pq_conn.send_query("select * from test".into()) {
        Ok(_) => loop {
            match pq_conn.is_result_ready() {
                Ok(true) => {
                    println!("send query succ");
                    pq_get_result(&mut pq_conn);
                    break;
                }
                Ok(false) => {
                    println!("send query ...");
                }
                Err(err) => {
                    println!("send query err:{}", err);
                    break;
                }
            }

            std::thread::sleep(Duration::from_millis(10));
        },
        Err(err) => println!("send_query err:{}", err),
    }
}


fn pq_get_result(pq_conn: &mut PqConn) {

    let pq_result = PqResult::new(pq_conn);// pq_conn.get_result();
    if pq_result.is_null() {
         println!(
        "pq conn status:{:?}  err msg:{}",
        pq_result.get_status(),pq_result.get_err_msg());
        return;
    }

    let rows = pq_result.get_rows();
    let cols = pq_result.get_clos();

    for row in 0..rows {
        for col in 0..cols {
            if !pq_result.is_null_val(row, col) {
                let val;
                let ty = pq_result.get_type(col);
                let len = pq_result.get_val_len(row, col);

                if ty == PqType::int4 as u32{
                    val = pq_result.get_i32_val(row, col);
                    println!("len:{} ty:{} get_i16_val:{} ", len, ty, val);

                }else{
                    let val = pq_result.get_string_val(row, col);
                    println!("len:{} ty:{} get_string_val:{} ", len, ty, val);
                }
            }
        }
    }
}
