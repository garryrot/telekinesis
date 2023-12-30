use std::sync::{Arc, Mutex};

use anyhow::Error;
use cxx::{CxxString, CxxVector};
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

pub struct ApiUpdate<State> {
    pub exec: fn(&mut State, i32, i32) -> bool
}

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

pub struct ApiBuilder<T> {
    pub init: ApiInit<T>,
    pub cmd: Vec<ApiCmd0<T>>,
    pub cmd_1: Vec<ApiCmd1<T>>,
    pub cmd_2: Vec<ApiCmd2<T>>,
    pub qry_str: Vec<ApiQryStr<T>>,
    pub qry_str_1: Vec<ApiQryStr1<T>>,
    pub qry_lst: Vec<ApiQryList<T>>,
    pub qry_lst_1: Vec<ApiQryList1<T>>,
    pub qry_bool: Vec<ApiQryBool<T>>,
    pub qry_bool_1: Vec<ApiQryBool1<T>>,
    pub control: Vec<ApiControl<T>>,
    pub update: ApiUpdate<T>,
    pub stop: ApiStop<T>,
}

impl<T> ApiBuilder<T> {
    pub fn new(init: ApiInit<T>) -> ApiBuilder<T> {
        ApiBuilder {
            init,
            cmd: vec![],
            cmd_1: vec![],
            cmd_2: vec![],
            qry_str: vec![],
            qry_str_1: vec![],
            qry_lst: vec![],
            qry_lst_1: vec![],
            qry_bool: vec![],
            qry_bool_1: vec![],
            control: vec![],
            update: ApiUpdate { exec: |_,_,_| false },
            stop: ApiStop { exec: |_,_| true },
        }
    }
    pub fn def_cmd(mut self, cmd: ApiCmd0<T>) -> Self {
        self.cmd.push(cmd);
        self
    }
    pub fn def_cmd1(mut self, cmd: ApiCmd1<T>) -> Self {
        self.cmd_1.push(cmd);
        self
    }
    pub fn def_cmd2(mut self, cmd: ApiCmd2<T>) -> Self {
        self.cmd_2.push(cmd);
        self
    }
    pub fn def_control(mut self, cmd: ApiControl<T>) -> Self {
        self.control.push(cmd);
        self
    }
    pub fn def_update(mut self, cmd: ApiUpdate<T>) -> Self {
        self.update = cmd;
        self
    }
    pub fn def_stop(mut self, cmd: ApiStop<T>) -> Self {
        self.stop = cmd;
        self
    }
    pub fn def_qry_str(mut self, cmd: ApiQryStr<T>) -> Self {
        self.qry_str.push(cmd);
        self
    }
    pub fn def_qry_str1(mut self, cmd: ApiQryStr1<T>) -> Self {
        self.qry_str_1.push(cmd);
        self
    }
    pub fn def_qry_lst(mut self, cmd: ApiQryList<T>) -> Self {
        self.qry_lst.push(cmd);
        self
    }
    pub fn def_qry_lst_1(mut self, cmd: ApiQryList1<T>) -> Self {
        self.qry_lst_1.push(cmd);
        self
    }
    pub fn def_qry_bool(mut self, cmd: ApiQryBool<T>) -> Self {
        self.qry_bool.push(cmd);
        self
    }
    pub fn def_qry_bool_1(mut self, cmd: ApiQryBool1<T>) -> Self {
        self.qry_bool_1.push(cmd);
        self
    }
}

pub trait Api<T> {
    fn state(&mut self) -> Arc<Mutex<Option<T>>>;
    fn fns(&self) -> ApiBuilder<T>;
    fn destroy(&mut self) -> ApiCmd0<T>;
    fn exec_cmd_0(&mut self, cmd: &str) -> bool {
        if cmd_matches(self.fns().init.name, cmd) {
            if let Ok(mut guard) = self.state().try_lock() {
                match (self.fns().init.exec)() {
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
                if guard.take().is_some() {
                    let api: ApiCmd0<T> = self.destroy();
                    return self.try_exec(|tk| (api.exec)(tk), false);
                }
            }
        }

        if let Some(api) = self.get_qry(self.fns().cmd, cmd) {
            return self.try_exec(|tk| (api.exec)(tk), false);
        }
        self.fail_dispatch(false)
    }

    fn exec_cmd_1(&mut self, cmd: &str, arg0: &str) -> bool {
        if let Some(api) = self.get_qry(self.fns().cmd_1, cmd) {
            return self.try_exec(|tk| (api.exec)(tk, arg0), false);
        }
        self.fail_dispatch(false)
    }

    fn exec_cmd_2(&mut self, cmd: &str, arg0: &str, arg1: &str) -> bool {
        if let Some(api) = self.get_qry(self.fns().cmd_2, cmd) {
            return self.try_exec(|tk| (api.exec)(tk, arg0, arg1), false);
        }
        self.fail_dispatch(false)
    }

    fn exec_qry_str(&mut self, qry: &str) -> String {
        if let Some(api) = self.get_qry(self.fns().qry_str, qry) {
            return self.try_exec(|tk| (api.exec)(tk), String::from(api.default));
        }
        self.fail_dispatch(String::new())
    }

    fn exec_qry_str_1(&mut self, qry: &str, arg0: &str) -> String {
        if let Some(api) = self.get_qry(self.fns().qry_str_1, qry) {
            return self.try_exec(|tk| (api.exec)(tk, arg0), String::from(api.default));
        }
        self.fail_dispatch(String::new())
    }

    fn exec_qry_lst(&mut self, qry: &str) -> Vec<String> {
        if let Some(api) = self.get_qry(self.fns().qry_lst, qry) {
            return self.try_exec(|tk| (api.exec)(tk), vec![]);
        }
        self.fail_dispatch(vec![])
    }

    fn exec_qry_lst_1(&mut self, qry: &str, arg0: &str) -> Vec<String> {
        if let Some(api) = self.get_qry(self.fns().qry_lst_1, qry) {
            return self.try_exec(|tk| (api.exec)(tk, arg0), vec![]);
        }
        self.fail_dispatch(vec![])
    }

    fn exec_qry_bool(&mut self, qry: &str) -> bool {
        if let Some(api) = self.get_qry(self.fns().qry_bool, qry) {
            return self.try_exec(|tk| (api.exec)(tk), false);
        }
        self.fail_dispatch(false)
    }

    fn exec_qry_bool_1(&mut self, qry: &str, arg0: &str) -> bool {
        if let Some(api) = self.get_qry(self.fns().qry_bool_1, qry) {
            return self.try_exec(|tk| (api.exec)(tk, arg0), false);
        }
        self.fail_dispatch(false)
    }

    fn exec_control(
        &mut self,
        qry: &str,
        arg0: i32,
        arg1: f32,
        arg2: &str,
        arg3: &CxxVector<CxxString>,
    ) -> i32 {
        let c = self.fns().control;
        if let Some(api) = self.get_qry(c, qry) {
            return self.try_exec(|tk| (api.exec)(tk, arg0, arg1, arg2, arg3), -1);
        }
        self.fail_dispatch(-1)
    }

    fn exec_update(&mut self, arg0: i32, arg1: i32) -> bool {
        let update = self.fns().update;  
        self.try_exec(|tk| (update.exec)(tk, arg0, arg1), false)
    }

    fn exec_stop(&mut self, arg0: i32) -> bool {
        let api = self.fns().stop;
        self.try_exec(|tk| (api.exec)(tk, arg0), false)
    }

    fn get_qry<ApiType>(&self, queries: Vec<ApiType>, name: &str) -> Option<ApiType>
    where
        ApiType: ApiImpl,
    {
        queries.into_iter().find(|q| cmd_matches(q.name(), name))
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
        fn fns(&self) -> ApiBuilder<EmptyState> {
            ApiBuilder::new(
                ApiInit {
                    name: "ctor",
                    exec: || Ok(EmptyState {}),
                }
            )
            .def_cmd(ApiCmd0 {
                name: "existing.command",
                exec: |_| true,
            })
            .def_cmd1(ApiCmd1 {
                name: "existing.command",
                exec: |_, _| true,
            })
            .def_qry_str(ApiQryStr {
                name: "existing.query",
                exec: |_| String::from("working"),
                default: "foobar",
            })
            .def_qry_str1(ApiQryStr1 {
                name: "existing.query",
                exec: |_, _| String::from("working"),
                default: "foobar",
            })
            .def_qry_lst(ApiQryList {
                name: "existing.query",
                exec: |_| vec![String::from("list0")],
            })
            .def_qry_lst_1(ApiQryList1 {
                name: "existing.query",
                exec: |_, _| vec![String::from("list1")],
            })
            .def_qry_bool(ApiQryBool {
                name: "existing.query",
                exec: |_| true,
            })
            .def_qry_bool_1(ApiQryBool1 {
                name: "existing.query",
                exec: |_, _| true,
            })
            .def_control(ApiControl {
                name: "vibrate",
                default: -1,
                exec: |_, _, _, _, _| 1,
            })
            .def_control(ApiControl {
                name: "move",
                default: -1,
                exec: |_, _, _, _, _| 2,
            })
        }
        fn state(&mut self) -> Arc<Mutex<Option<EmptyState>>> {
            self.state.clone()
        }
        fn destroy(&mut self) -> ApiCmd0<EmptyState> {
            ApiCmd0 {
                name: "dtor",
                exec: |_| false,
            }
        }
    }

    #[test]
    fn test_api() {
        // enable_log();
        let mut api = TestApi::new();
        assert_eq!(api.exec_qry_str("existing.query"), String::from("foobar"));
        api.exec_cmd_0("ctor");
        assert!(! api.exec_cmd_0("non.existing.command"));
        assert!(api.exec_cmd_0("existing.command"));
        assert!(api.exec_cmd_0("Existing.command"));
        assert!(! api.exec_cmd_1("non.existing.command", "some_arg"));
        assert!(api.exec_cmd_1("existing.command", "some_arg"));
        assert!(api.exec_cmd_1("existing.command", "some_arg"));
        assert_eq!(api.exec_qry_str("existing.query"), String::from("working"));
        assert_eq!(api.exec_qry_lst("existing.query")[0], "list0");
        assert_eq!(api.exec_qry_lst_1("existing.query", "p")[0], "list1");
        assert!(api.exec_qry_bool("existing.query"));
        assert!(api.exec_qry_bool_1("existing.query", "something"));
    }
}
