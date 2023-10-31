use std::sync::{Arc, Mutex};

use anyhow::Error;
use cxx::{CxxVector, CxxString};
use tracing::error;

macro_rules! declare_api_cmd {
    ($t:ident) => {
        impl<T> ApiImpl for $t<T> {
            fn name(&self) -> &'static str {
                self.name
            }
        }
    };
}

pub trait ApiImpl {
    fn name(&self) -> &'static str;
}

pub struct ApiInit<State> {
    pub name: &'static str,
    pub exec: fn() -> Result<State, Error>,
}
declare_api_cmd!(ApiInit);

pub struct ApiCmd0<State> {
    pub name: &'static str,
    pub exec: fn(&mut State) -> bool,
}
declare_api_cmd!(ApiCmd0);

pub struct ApiCmd1<State> {
    pub name: &'static str,
    pub exec: fn(&mut State, &str) -> bool,
}
declare_api_cmd!(ApiCmd1);

pub struct ApiCmd2<State> {
    pub name: &'static str,
    pub exec: fn(&mut State, &str, &str) -> bool,
}
declare_api_cmd!(ApiCmd2);

pub struct ApiQryStr<State> {
    pub name: &'static str,
    pub default: &'static str,
    pub exec: fn(&mut State) -> String,
}
declare_api_cmd!(ApiQryStr);

pub struct ApiQryStr1<State> {
    pub name: &'static str,
    pub default: &'static str,
    pub exec: fn(&mut State, &str) -> String,
}
declare_api_cmd!(ApiQryStr1);

pub struct ApiQryBool<State> {
    pub name: &'static str,
    pub exec: fn(&mut State) -> bool,
}
declare_api_cmd!(ApiQryBool);

pub struct ApiQryBool1<State> {
    pub name: &'static str,
    pub exec: fn(&mut State, &str) -> bool,
}
declare_api_cmd!(ApiQryBool1);

pub struct ApiControl<State> {
    pub name: &'static str,
    pub exec: fn(&mut State, i32, f32, &str, &CxxVector<CxxString>) -> i32,
    pub default: i32,
}
declare_api_cmd!(ApiControl);

pub struct ApiStop<State> {
    pub exec: fn(&mut State, i32) -> bool,
}

pub struct ApiQryList<State> {
    pub name: &'static str,
    pub exec: fn(&mut State) -> Vec<String>,
}
impl<State> ApiImpl for ApiQryList<State> {
    fn name(&self) -> &'static str {
        self.name
    }
}

pub struct ApiQryList1<State> {
    pub name: &'static str,
    pub exec: fn(&mut State, &str) -> Vec<String>,
}
impl<State> ApiImpl for ApiQryList1<State> {
    fn name(&self) -> &'static str {
        self.name
    }
}

fn cmd_matches(str1: &str, cmd: &str) -> bool {
    str1.to_lowercase() == cmd.to_lowercase()
}

pub trait Api<T> {
    fn state(&mut self) -> Arc<Mutex<Option<T>>>;
    fn destroy(&mut self) -> ApiCmd0<T>;
    fn init(&self) -> ApiInit<T>;
    fn cmd_0(&self) -> Vec<ApiCmd0<T>>;

    fn exec_cmd_0(&mut self, cmd: &str) -> bool {
        if cmd_matches(self.init().name, cmd) {
            if let Ok(mut guard) = self.state().try_lock() {
                match (self.init().exec)() {
                    Ok(state) => {
                        guard.replace(state);
                    }
                    Err(err) => {
                        error!("error constructing state: {:?}", err);
                    }
                }
            }
        }

        if cmd_matches(self.destroy().name, cmd) {
            if let Ok(mut guard) = self.state().try_lock() {
                if let Some(_) = guard.take() {
                    let api: ApiCmd0<T> = self.destroy();
                    return self.try_exec(|tk| (api.exec)(tk), false);
                }
            }
        }

        if let Some(api) = self.get_qry(self.cmd_0(), cmd) {
            return self.try_exec(|tk| (api.exec)(tk), false);
        }
        self.fail_dispatch(false)
    }

    fn cmd_1(&self) -> Vec<ApiCmd1<T>>;
    fn exec_cmd_1(&mut self, cmd: &str, arg0: &str) -> bool {
        if let Some(api) = self.get_qry(self.cmd_1(), cmd) {
            return self.try_exec(|tk| (api.exec)(tk, arg0), false);
        }
        self.fail_dispatch(false)
    }

    fn cmd_2(&self) -> Vec<ApiCmd2<T>>;
    fn exec_cmd_2(&mut self, cmd: &str, arg0: &str, arg1: &str) -> bool {
        if let Some(api) = self.get_qry(self.cmd_2(), cmd) {
            return self.try_exec(|tk| (api.exec)(tk, arg0, arg1), false);
        }
        self.fail_dispatch(false)
    }

    fn qry_str(&self) -> Vec<ApiQryStr<T>>;
    fn exec_qry_str(&mut self, qry: &str) -> String {
        if let Some(api) = self.get_qry(self.qry_str(), qry) {
            return self.try_exec(|tk| (api.exec)(tk), String::from(api.default));
        }
        self.fail_dispatch(String::from(""))
    }

    fn qry_str_1(&self) -> Vec<ApiQryStr1<T>>;
    fn exec_qry_str_1(&mut self, qry: &str, arg0: &str) -> String {
        if let Some(api) = self.get_qry(self.qry_str_1(), qry) {
            return self.try_exec(|tk| (api.exec)(tk, arg0), String::from(api.default));
        }
        self.fail_dispatch(String::from(""))
    }

    fn qry_lst(&self) -> Vec<ApiQryList<T>>;
    fn exec_qry_lst(&mut self, qry: &str) -> Vec<String> {
        if let Some(api) = self.get_qry(self.qry_lst(), qry) {
            return self.try_exec(|tk| (api.exec)(tk), vec![]);
        }
        self.fail_dispatch(vec![])
    }

    fn qry_lst_1(&self) -> Vec<ApiQryList1<T>>;
    fn exec_qry_lst_1(&mut self, qry: &str, arg0: &str) -> Vec<String> {
        if let Some(api) = self.get_qry(self.qry_lst_1(), qry) {
            return self.try_exec(|tk| (api.exec)(tk, arg0), vec![]);
        }
        self.fail_dispatch(vec![])
    }

    fn qry_bool(&self) -> Vec<ApiQryBool<T>>;
    fn exec_qry_bool(&mut self, qry: &str) -> bool {
        if let Some(api) = self.get_qry(self.qry_bool(), qry) {
            return self.try_exec(|tk| (api.exec)(tk), false);
        }
        self.fail_dispatch(false)
    }

    fn qry_bool_1(&self) -> Vec<ApiQryBool1<T>>;
    fn exec_qry_bool_1(&mut self, qry: &str, arg0: &str) -> bool {
        if let Some(api) = self.get_qry(self.qry_bool_1(), qry) {
            return self.try_exec(|tk| (api.exec)(tk, arg0), false);
        }
        self.fail_dispatch(false)
    }

    fn control(&self) -> Vec<ApiControl<T>>;
    fn exec_control(&mut self, qry: &str, arg0: i32, arg1: f32, arg2: &str, arg3: &CxxVector<CxxString>) -> i32 {
        if let Some(api) = self.get_qry(self.control(), qry) {
            return self.try_exec(|tk| (api.exec)(tk, arg0, arg1, arg2, arg3), -1);
        }
        self.fail_dispatch(-1)
    }

    fn stop(&self) -> ApiStop<T>;
    fn exec_stop(&mut self, arg0: i32) -> bool {
        let api = self.stop();
        self.try_exec(|tk| (api.exec)(tk, arg0), false)
    }

    fn get_qry<ApiType>(&mut self, queries: Vec<ApiType>, name: &str) -> Option<ApiType>
    where
        ApiType: ApiImpl,
    {
        for q in queries {
            if cmd_matches(q.name(), name) {
                return Some(q);
            }
        }
        None
    }

    fn try_exec<F, R>(&mut self, func: F, default: R) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let tele = &self.state();

        if let Ok(mut guard) = tele.try_lock() {
            match guard.take() {
                Some(mut tk) => {
                    let result = func(&mut tk);
                    guard.replace(tk);
                    return result;
                }
                None => error!("Dispatch on 'None'"),
            }
        } else {
            error!("Failed locking mutex");
        }
        default
    }

    fn fail_dispatch<D>(&self, default: D) -> D {
        error!("Cmd not found");
        default
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    struct EmptyState {}

    struct TestApi {
        pub state: Arc<Mutex<Option<EmptyState>>>,
    }

    impl TestApi {
        pub fn new() -> Self {
            TestApi {
                state: Arc::new(Mutex::new(None)),
            }
        }
    }

    impl Api<EmptyState> for TestApi {
        fn init(&self) -> ApiInit<EmptyState> {
            ApiInit {
                name: "ctor",
                exec: || Ok(EmptyState {}),
            }
        }

        fn cmd_0(&self) -> Vec<ApiCmd0<EmptyState>> {
            vec![ApiCmd0 {
                name: "existing.command",
                exec: |_| true,
            }]
        }

        fn cmd_1(&self) -> Vec<ApiCmd1<EmptyState>> {
            vec![ApiCmd1 {
                name: "existing.command",
                exec: |_, _| true,
            }]
        }

        fn qry_str(&self) -> Vec<ApiQryStr<EmptyState>> {
            vec![ApiQryStr {
                name: "existing.query",
                exec: |_| String::from("working"),
                default: "foobar",
            }]
        }

        fn qry_str_1(&self) -> Vec<ApiQryStr1<EmptyState>> {
            vec![ApiQryStr1 {
                name: "existing.query",
                exec: |_, _| String::from("working"),
                default: "foobar",
            }]
        }

        fn qry_lst(&self) -> Vec<ApiQryList<EmptyState>> {
            vec![ApiQryList {
                name: "existing.query",
                exec: |_| vec![String::from("list0")],
            }]
        }

        fn qry_lst_1(&self) -> Vec<ApiQryList1<EmptyState>> {
            vec![ApiQryList1 {
                name: "existing.query",
                exec: |_, _| vec![String::from("list1")],
            }]
        }
        
        fn qry_bool(&self) -> Vec<ApiQryBool<EmptyState>> {
            vec![ApiQryBool {
                name: "existing.query",
                exec: |_| true,
            }]
        }

        fn qry_bool_1(&self) -> Vec<ApiQryBool1<EmptyState>> {
            vec![ApiQryBool1 {
                name: "existing.query",
                exec: |_,_| true,
            }]
        }

        fn state(&mut self) -> Arc<Mutex<Option<EmptyState>>> {
            self.state.clone()
        }

        fn control(&self) -> Vec<ApiControl<EmptyState>> {
            vec![ApiControl {
                name: "vibrate",
                default: -1,
                exec: |_,_,_,_,_| 1,
            },
            ApiControl {
                name: "move",
                default: -1,
                exec: |_,_,_,_,_| 2,
            }]
        }
        
        fn stop(&self) -> ApiStop<EmptyState> {
            ApiStop { exec: |_,_| true }
        }

        fn destroy(&mut self) -> ApiCmd0<EmptyState> {
            ApiCmd0 {
                name: "dtor",
                exec: |_| false,
            }
        }

        fn cmd_2(&self) -> Vec<ApiCmd2<EmptyState>> {
            vec![]
        }
    }

    #[test]
    fn test_api() {
        // enable_log();
        let mut api = TestApi::new();
        assert_eq!(api.exec_qry_str("existing.query"), String::from("foobar"));
        // assert_eq!(api.exec_control("vibrate", 100, 2.5, "pattern", vec![]), -1);
        api.exec_cmd_0("ctor");
        assert_eq!(api.exec_cmd_0("non.existing.command"), false);
        assert_eq!(api.exec_cmd_0("existing.command"), true);
        assert_eq!(api.exec_cmd_0("Existing.command"), true);
        assert_eq!(api.exec_cmd_1("non.existing.command", "some_arg"), false);
        assert_eq!(api.exec_cmd_1("existing.command", "some_arg"), true);
        assert_eq!(api.exec_cmd_1("existing.command", "some_arg"), true);
        assert_eq!(api.exec_qry_str("existing.query"), String::from("working"));
        assert_eq!(api.exec_qry_lst("existing.query")[0], "list0");
        assert_eq!(api.exec_qry_lst_1("existing.query", "p")[0], "list1");
        assert_eq!(api.exec_qry_bool("existing.query"), true);
        assert_eq!(api.exec_qry_bool_1("existing.query", "something"), true);
        // assert_eq!(api.exec_control("vibrate", 100, 2.5, "pattern", vec![]), 1);
        // assert_eq!(api.exec_control("move", 100, 2.5, "pattern", vec![]), 2);
    }
}
