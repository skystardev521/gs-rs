mod bindings;
use bindings::{
    ConnStatusType, PGconn, PGresult, PQclear, PQcmdTuples, PQconnectdb, PQconsumeInput,
    PQerrorMessage, PQescapeByteaConn, PQescapeStringConn, PQfinish, PQfname, PQftype, PQgetResult,
    PQgetisnull, PQgetlength, PQgetvalue, PQisBusy, PQresultStatus, PQsendQuery, PQsetnonblocking,
    PQstatus, PQunescapeBytea, CONNECTION_OK,
};

pub mod op;
use op::TOpResp;
pub mod opdb;

pub mod db_conn;
pub mod op_base;
pub mod op_queue;
pub mod bytes;
pub mod test;
pub mod pq_result;
pub(crate) mod pq_conn;

use std::ptr;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

struct ConnMgmt {
    //空闲中
    idle_vec: Vec<Conn>,
    //忙碌中
    busy_vec: Vec<Conn>,
    //断开中
    //disc_vec:  Vec<Conn>,
}

struct Conn {
    id: u32,
    pg_conn: *mut PGconn,
    conn_str: String,
    tarit_op: Option<Box<dyn TOpResp>>,
}

struct PGResult {
    result: *mut PGresult,
}
impl Drop for PGResult {
    fn drop(&mut self) {
        if !self.result.is_null() {
            unsafe {
                PQclear(self.result);
            }
        }
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        if !self.pg_conn.is_null() {
            unsafe {
                PQfinish(self.pg_conn);
            }
        }
    }
}

#[inline]
fn c_char_ptr_to_string(c_char_ptr: *const c_char) -> String {
    unsafe { CStr::from_ptr(c_char_ptr).to_string_lossy().to_string() }
}

#[inline]
fn pg_error_message(pg_conn: *const PGconn) -> String {
    unsafe { c_char_ptr_to_string(PQerrorMessage(pg_conn)) }
}

#[inline]
fn pq_clear(res: *mut PGresult) {
    if !res.is_null() {
        unsafe { PQclear(res) }
    }
}

#[inline]
fn pq_status(pg_conn: *mut PGconn) -> ConnStatusType {
    unsafe { PQstatus(pg_conn) }
}

impl Conn {
    pub fn new(id: u32, conn_str: String) -> Result<Self, String> {
        match CString::new(conn_str.clone()) {
            Ok(cstr) => {
                let pg_conn = unsafe { PQconnectdb(cstr.as_ptr()) };
                if pq_status(pg_conn) == CONNECTION_OK {
                    unsafe { PQsetnonblocking(pg_conn, 1) };
                    return Ok(Conn {
                        id,
                        conn_str,
                        pg_conn,
                        tarit_op: None,
                    });
                } else {
                    let err = pg_error_message(pg_conn);
                    if !pg_conn.is_null() {
                        unsafe { PQfinish(pg_conn) };
                    }
                    return Err(format!("{} error:{}", conn_str, err));
                }
            }
            Err(nul_err) => {
                return Err(format!(
                    "conn str:{} to CString Error:{}",
                    conn_str,
                    nul_err.to_string()
                ))
            }
        }
    }

    pub fn pq_send_query(&self, sql: String) -> Result<bool, String> {
        match CString::new(sql) {
            Ok(csql) => {
                let ret = unsafe { PQsendQuery(self.pg_conn, csql.as_ptr()) };
                if ret == 0 {
                    return Err(pg_error_message(self.pg_conn));
                }
                return Ok(true);
            }
            Err(nul_err) => return Err(format!("sql error:{}", nul_err.to_string())),
        }
    }
    pub fn pq_get_reesult(&self) {
        unsafe {
            PQgetResult(self.pg_conn);
        }
    }
}

//---------------------------test code ---------------------------------
