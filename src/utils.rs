macro_rules! global {
    ($name:ident, $type:ty, $default:expr) => {
        pub fn $name() -> &'static mut $type {
            static mut VALUE: Option<$type> = None;
            unsafe { VALUE.get_or_insert_with(|| $default) }
        }
    };
}
macro_rules! global_init {
    ($name:ident, $type:ty, $initializer:expr) => {
        pub fn $name() -> &'static mut $type {
            static mut VALUE: Option<$type> = None;
            unsafe { VALUE.get_or_insert_with($initializer) }
        }
    };
}
macro_rules! global_init_async {
    ($name:ident, $type:ty, $initializer:expr) => {
        pub async fn $name() -> &'static mut $type {
            static mut VALUE: Option<$type> = None;
            unsafe {
                if VALUE.is_none() {
                    let value = $initializer().await;
                    VALUE.replace(value);
                }
                VALUE.as_mut().unwrap()
            }
        }
    };
}

macro_rules! early_continue {
    ($condition:expr) => {
        if $condition {
            continue;
        }
    };
}

macro_rules! early_return {
    ($condition:expr) => {
        if $condition {
            return;
        }
    };
}

macro_rules! early_return_value {
    ($condition:expr, $value:expr) => {
        if $condition {
            return $value;
        }
    };
}

pub(crate) use early_continue;
pub(crate) use early_return;
pub(crate) use early_return_value;
pub(crate) use global;
pub(crate) use global_init;
pub(crate) use global_init_async;
