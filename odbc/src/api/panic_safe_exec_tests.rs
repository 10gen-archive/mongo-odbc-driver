use crate::trace_call_and_outcome;
use crate::{
    errors::ODBCError,
    handles::definitions::{MongoHandle, MongoHandleRef, Statement, StatementState},
    panic_safe_exec,
};
use ::function_name::named;
use file_dbg_macros::dbg_write;
use odbc_sys::{HStmt, SqlReturn};
use std::{panic, sync::mpsc};

mod unit {
    use super::*;
    use lazy_static::lazy_static;
    use regex::{Regex, RegexBuilder};

    lazy_static! {
        static ref PANIC_ERROR_MSG: Regex = RegexBuilder::new("panic")
            .case_insensitive(true)
            .build()
            .unwrap();
    }

    #[named]
    fn non_panic_fn(stmt_handle: HStmt) -> SqlReturn {
        panic_safe_exec!(|| { SqlReturn::SUCCESS }, stmt_handle);
    }

    #[named]
    fn panic_fn(stmt_handle: HStmt) -> SqlReturn {
        panic_safe_exec!(|| { panic!("panic test") }, stmt_handle);
    }

    #[test]
    fn test_non_panic() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        let sql_return = non_panic_fn(stmt_handle as *mut _);
        assert_eq!(SqlReturn::SUCCESS, sql_return);
        unsafe {
            assert!((*stmt_handle)
                .as_statement()
                .unwrap()
                .errors
                .read()
                .unwrap()
                .is_empty());
        }
    }

    #[test]
    fn test_panic() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        let sql_return = panic_fn(stmt_handle as *mut _);
        assert_eq!(SqlReturn::ERROR, sql_return);
        unsafe {
            let actual_error = format!(
                "{:?}",
                (*stmt_handle)
                    .as_statement()
                    .unwrap()
                    .errors
                    .read()
                    .unwrap()[0]
            );

            // Using a substring of the error because directory format differs for windows and linux
            // and to not depend on line number.
            assert!(
                PANIC_ERROR_MSG.is_match(actual_error.as_str()),
                "Expected an error due to panic, but got {}",
                &actual_error
            );
        }
    }
}
