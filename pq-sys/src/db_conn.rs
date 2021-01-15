
/*
use bindings::{
    ConnStatusType, PGconn, PGresult, PQclear, PQcmdTuples, PQconnectdb, PQconsumeInput,
    PQerrorMessage, PQescapeByteaConn, PQescapeStringConn, PQfinish, PQfname, PQftype, PQgetResult,
    PQgetisnull, PQgetlength, PQgetvalue, PQisBusy, PQresultStatus, PQsendQuery, PQsetnonblocking,
    PQstatus, PQunescapeBytea, CONNECTION_OK,
};

use op_base::IReqResp;
use op_base::OpBase;
use op_base::State;
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::ptr::slice_from_raw_parts;

use crate::bindings::PQreset;

pub struct Conn {
    conn_str: String,
    conn_ptr: Option<*mut PGconn>,
    queue: VecDeque<Box<dyn IReqResp>>,
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
    pub fn new(conn_str: String) -> Self {
        Conn {
            conn_str,
            conn_ptr: None,
            queue: VecDeque::new(),
        }
    }

    pub fn conn(&mut self) -> Result<(), String> {
        match CString::new(self.conn_str.clone()) {
            Ok(cstr) => {
                let pg_conn = unsafe { PQconnectdb(cstr.as_ptr()) };
                if pq_status(pg_conn) == CONNECTION_OK {
                    unsafe { PQsetnonblocking(pg_conn, 1) };
                    self.conn_ptr = Some(pg_conn);
                    return Ok(());
                } else {
                    let err = pg_error_message(pg_conn);
                    if !pg_conn.is_null() {
                        unsafe { PQfinish(pg_conn) };
                    }
                    return Err(format!("{} error:{}", self.conn_str, err));
                }
            }
            Err(nul_err) => {
                return Err(format!(
                    "conn str:{} to CString Error:{}",
                    self.conn_str,
                    nul_err.to_string()
                ))
            }
        }
    }

    pub fn reconn(&mut self) -> bool {
        if let Some(ptr) = self.conn_ptr {
            unsafe { PQreset(ptr) }
        }
        return true;
    }

    pub fn get_queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn get_conn_ptr(&self) -> Option<*mut PGconn> {
        self.conn_ptr
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

impl Drop for Conn {
    fn drop(&mut self) {
        if let Some(ptr) = self.conn_ptr {
            unsafe {
                PQfinish(ptr);
            }
        }
    }
}

#[test]
fn test() {}
*/