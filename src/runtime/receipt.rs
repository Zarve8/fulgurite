use solana_program::{
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey
};
use crate::runtime::utils;

#[derive(Debug)]
pub struct Receipt {
    pub result: ProgramResult,
    pub(crate) log_messages: Vec<String>,
    pub(crate) log_datas: Vec<(Pubkey, Vec<Vec<u8>>)>,
    pub(crate) call_stack: Vec<Pubkey>,
    pub(crate) return_data: Option<(Pubkey, Vec<u8>)>,
}

impl Receipt {
    pub(crate) fn new() -> Self {
        Self {
            result: Ok(()),
            log_messages: Vec::new(),
            log_datas: Vec::new(),
            call_stack: Vec::new(),
            return_data: None,
        }
    }

    pub(crate) fn push_msg(&mut self, msg: String) {
        println!("{}", &msg);
        self.log_messages.push(msg);
    }

    pub(crate) fn active_program(&self) -> Pubkey {
        self.call_stack.last().unwrap().clone()
    }

    pub(crate) fn log_program_invoked(&mut self, program: &Pubkey) {
        self.push_msg(format!("Program invoked: {}", program.to_string()));
    }

    pub(crate) fn log_program_succeed(&mut self) {
        self.push_msg("Program consumed: 0 of 200000 compute units".to_string());
        self.push_msg("Program returned success".to_string());
    }

    pub(crate) fn log_program_failed(&mut self, err: ProgramError) {
        self.push_msg(format!("Program returned error: \"{:?}\"", err));
    }
}

// +++++++++ Suit Methods +++++++++
impl Receipt {
    pub fn expect_ok(&self) {
        if self.result.is_err() {
            println!("Invoke failed with {:?}", self.result.clone().err().unwrap());
            assert_eq!(self.result, Ok(()));
        }
    }

    pub fn expect_err(&self, err: ProgramError) {
        if self.result.is_ok() {
            println!("Invoke succeed when must not");
            assert_ne!(self.result, Ok(()));
        }

        let got_err = self.result.clone().err().unwrap();
        if !got_err.eq(&err) {
            println!("Invoke failed with different error {:?} != {:?}", got_err, err);
            assert_eq!(got_err, err);
        }
    }

    pub fn expect_any_err(&self) {
        if self.result.is_ok() {
            println!("Invoke succeed when must not");
            assert_ne!(self.result, Ok(()));
        }
    }

    pub fn expect_log(&self, msg: &str) {
        if !self.contains_program_log(msg) {
            println!("Log not found {}", msg);
            assert!(false);
        }
    }

    pub fn expect_data(&self, program_id: &Pubkey, data: &[&[u8]]) {
        if !self.contains_data(program_id, data) {
            println!("Data log not found {}: {:?}", program_id.to_string(), data);
            assert!(false);
        }
    }
    
    fn contains_program_log(&self, text: &str) -> bool {
        self.contains_log(&format!("Program logged: \"{}\"", text))
    }

    fn contains_log(&self, text: &str) -> bool {
        for log in self.log_messages.iter() {
            if log.eq(text) {
                return true;
            }
        }

        false
    }

    fn contains_data(&self, program: &Pubkey, data: &[&[u8]]) -> bool {
        for (log_key, log_data) in self.log_datas.iter() {
            if log_key.eq(program) {
                let mut matched = log_data.len() == data.len();
                if !matched {
                    continue;
                }

                for (a, b) in log_data.iter().zip(data.iter()) {
                    matched &= utils::compare_arrays(a.as_slice(), *b);
                }
                if matched {
                    return true;
                }
            }
        }

        false
    }
}