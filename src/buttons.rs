pub mod prelude {
    pub use crate::_button_interrupt_handler;
    pub use crate::_button_interrupt_handler_iter;
    pub use crate::button_interrupt;
    pub use paste::paste;
}

#[macro_export]
macro_rules! _button_interrupt_handler {
    ($controller_res:ident, $event_handler:ident, $task_name: ident, $intr: ident, $ctrl_action: ident) => {
        #[task(binds = $intr, resources = [$controller_res])]
        fn $task_name(mut cx: $task_name::Context) {
            if let Some(event) = cx.resources.$controller_res.lock(|ctl| ctl.$ctrl_action()) {
                $event_handler::spawn(event).ok();
            }
        }
    };
}

#[macro_export]
macro_rules! _button_interrupt_handler_iter {
    ($controller_res:ident, $event_handler:ident, [ $($id:expr),+ ] ) => {
        paste! {
            $(
                _button_interrupt_handler!($controller_res, $event_handler, [<_btn_intr_ $id>],
                        [<EIC_EXTINT_ $id>], [<interrupt_extint $id>]);
            )*
        }
    }
}

#[macro_export]
macro_rules! button_interrupt {
    ($controller_res:ident, $event_handler:ident) => {
        _button_interrupt_handler_iter!($controller_res, $event_handler, [3, 4, 5, 7, 10, 11, 12]);
    };
}
