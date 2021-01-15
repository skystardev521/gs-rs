

/// | Rust type                         | Postgres type(s)                              |
/// |-----------------------------------|-----------------------------------------------|
/// | `bool`                            | BOOL                                          |
/// | `i8`                              | "char"                                        |
/// | `i16`                             | SMALLINT, SMALLSERIAL                         |
/// | `i32`                             | INT, SERIAL                                   |
/// | `u32`                             | OID                                           |
/// | `i64`                             | BIGINT, BIGSERIAL                             |
/// | `f32`                             | REAL                                          |
/// | `f64`                             | DOUBLE PRECISION                              |
/// | `&str`/`String`                   | VARCHAR, CHAR(n), TEXT, CITEXT, NAME, UNKNOWN |
/// | `&[u8]`/`Vec<u8>`                 | BYTEA                                         |
/// | `HashMap<String, Option<String>>` | HSTORE                                        |
/// | `SystemTime`                      | TIMESTAMP, TIMESTAMP WITH TIME ZONE           |
/// | `IpAddr`                          | INET                                          |

use bindings::{
    ExecStatusType, PGresult, PQresultStatus, PQclear, PQcmdTuples, 
    PQfname, PQfnumber,  PQftype,  PQgetisnull,PQgetlength, PQgetvalue, PQnfields, PQntuples,
};

use pq_conn::PqConn;

use std::ffi::CStr;
use std::str::FromStr;

/// len:-1(不定长度)
pub enum PqType {
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
pub struct PqResult<'a> {
    res: *mut PGresult,
    conn: &'a mut PqConn,
}

impl<'a> Drop for PqResult<'a> {
    fn drop(&mut self) {
        self.clear();
        self.conn.set_available(true)
    }
}

impl<'a> PqResult<'a> {
    pub fn new(conn: &'a mut PqConn) -> Self {
        let res = conn.get_result();
        PqResult { conn, res }
    }

    fn clear(&self){
        if !self.res.is_null() {
            unsafe {
                PQclear(self.res);
            }
        }
    }

    /// 是否有结果集
    pub fn is_null(&self) -> bool {
        self.res.is_null()
    }

    /// 一次运行多条命令时使用
    /// return:false 没有结果
    pub fn next(&mut self)->bool{
        self.clear();
        self.res = self.conn.get_result();
        
        !self.res.is_null() == true
    }

    /// 获取错误信息
    #[inline(always)]
    pub fn get_err_msg(&self) -> String {
        self.conn.get_err_msg()
    }

    #[inline(always)]
    pub fn get_rows(&self) -> i32 {
        unsafe { PQntuples(self.res) }
    }
    #[inline(always)]
    pub fn get_clos(&self) -> i32 {
        unsafe { PQnfields(self.res) }
    }

    #[inline(always)]
    pub fn get_status(&self) -> ExecStatusType {
        unsafe { PQresultStatus(self.res) }
    }

    #[inline(always)]
    pub fn get_type(&self, col: i32) -> u32 {
        unsafe { PQftype(self.res, col) }
    }

    /// 根据列名获取 数据所在的列
    #[inline(always)]
    pub fn get_col(&self, fname: &str) -> i32 {
        unsafe { PQfnumber(self.res, fname.as_ptr() as *const i8) }
    }

    /// 反回 当前 行,列 的数据长度
    #[inline(always)]
    pub fn get_val_len(&self, row: i32, col: i32) -> i32 {
        unsafe { PQgetlength(self.res, row, col) }
    }

    #[inline(always)]
    pub fn get_fname(&self, col: i32) -> *mut ::std::os::raw::c_char {
        unsafe { PQfname(self.res, col) }
    }
    /// 反回 当前 行,列 的数据长度
    #[inline(always)]
    pub fn get_val(&self, row: i32, col: i32) -> *mut ::std::os::raw::c_char {
        unsafe { PQgetvalue(self.res, row, col) }
    }

    /// false:non null true:null
    #[inline(always)]
    pub fn is_null_val(&self, row: i32, col: i32) -> bool {
        unsafe { PQgetisnull(self.res, row, col) == 1 }
    }

    /// 返回被 SQL 命令影响的行的数量
    #[inline(always)]
    pub fn get_cmd_rows(&self) -> *mut ::std::os::raw::c_char {
        unsafe { PQcmdTuples(self.res) }
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